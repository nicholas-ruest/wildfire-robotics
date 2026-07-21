//! Canonical subject grammar and default-deny least-privilege authorization.

use std::collections::BTreeSet;
use thiserror::Error;

const TOKEN_MAX_BYTES: usize = 128;
const SUBJECT_MAX_BYTES: usize = 512;

/// Broker operation governed by a subject grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    /// Publish a message.
    Publish,
    /// Subscribe or create a consumer.
    Subscribe,
}

/// Parsed canonical integration subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subject {
    canonical: String,
    environment: String,
    region: String,
    tenant: String,
    context: String,
    aggregate: String,
    event: String,
    major: u16,
}

impl Subject {
    /// Parses `wr.environment.region.tenant.context.aggregate.event.v<major>`.
    pub fn parse(value: &str) -> Result<Self, SubjectError> {
        if value.is_empty() || value.len() > SUBJECT_MAX_BYTES || !value.is_ascii() {
            return Err(SubjectError::InvalidSubject);
        }
        let parts: Vec<_> = value.split('.').collect();
        if parts.len() != 8 || parts[0] != "wr" {
            return Err(SubjectError::InvalidSubject);
        }
        for token in &parts[1..7] {
            validate_token(token)?;
        }
        let major = parts[7]
            .strip_prefix('v')
            .filter(|number| !number.is_empty() && !number.starts_with('0'))
            .and_then(|number| number.parse::<u16>().ok())
            .filter(|major| *major > 0)
            .ok_or(SubjectError::InvalidSubject)?;
        Ok(Self {
            canonical: value.into(),
            environment: parts[1].into(),
            region: parts[2].into(),
            tenant: parts[3].into(),
            context: parts[4].into(),
            aggregate: parts[5].into(),
            event: parts[6].into(),
            major,
        })
    }

    /// Canonical transport subject.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }
    /// Environment boundary.
    #[must_use]
    pub fn environment(&self) -> &str {
        &self.environment
    }
    /// Region boundary.
    #[must_use]
    pub fn region(&self) -> &str {
        &self.region
    }
    /// Tenant boundary.
    #[must_use]
    pub fn tenant(&self) -> &str {
        &self.tenant
    }
    /// Owning bounded context.
    #[must_use]
    pub fn context(&self) -> &str {
        &self.context
    }
    /// Aggregate subject token.
    #[must_use]
    pub fn aggregate(&self) -> &str {
        &self.aggregate
    }
    /// Event subject token.
    #[must_use]
    pub fn event(&self) -> &str {
        &self.event
    }
    /// Contract major version.
    #[must_use]
    pub const fn major(&self) -> u16 {
        self.major
    }
}

fn validate_token(value: &str) -> Result<(), SubjectError> {
    if value.is_empty()
        || value.len() > TOKEN_MAX_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        || matches!(value, "*" | ">")
    {
        Err(SubjectError::InvalidSubject)
    } else {
        Ok(())
    }
}

/// Exact least-privilege grant; wildcards are intentionally inexpressible.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubjectGrant {
    action: Action,
    environment: String,
    region: String,
    tenant: String,
    context: String,
    events: BTreeSet<String>,
}

impl SubjectGrant {
    /// Creates a bounded grant for exact authority boundaries and event names.
    pub fn new<I, S>(
        action: Action,
        environment: &str,
        region: &str,
        tenant: &str,
        context: &str,
        events: I,
    ) -> Result<Self, SubjectError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for value in [environment, region, tenant, context] {
            validate_token(value)?;
        }
        let events: BTreeSet<_> = events.into_iter().map(Into::into).collect();
        if events.is_empty() || events.iter().any(|event| validate_token(event).is_err()) {
            return Err(SubjectError::InvalidGrant);
        }
        Ok(Self {
            action,
            environment: environment.into(),
            region: region.into(),
            tenant: tenant.into(),
            context: context.into(),
            events,
        })
    }

    fn permits(&self, action: Action, subject: &Subject) -> bool {
        self.action == action
            && self.environment == subject.environment
            && self.region == subject.region
            && self.tenant == subject.tenant
            && self.context == subject.context
            && self.events.contains(&subject.event)
    }
}

/// Default-deny collection of exact subject grants.
#[derive(Clone, Debug, Default)]
pub struct SubjectAuthorizer(Vec<SubjectGrant>);

impl SubjectAuthorizer {
    /// Creates an authorizer from reviewed grants.
    #[must_use]
    pub fn new(grants: impl IntoIterator<Item = SubjectGrant>) -> Self {
        Self(grants.into_iter().collect())
    }
    /// Authorizes an exact parsed subject or denies before broker I/O.
    pub fn authorize(&self, action: Action, subject: &Subject) -> Result<(), SubjectError> {
        if self.0.iter().any(|grant| grant.permits(action, subject)) {
            Ok(())
        } else {
            Err(SubjectError::Unauthorized)
        }
    }
}

/// Stable subject grammar and authorization failures.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum SubjectError {
    /// Subject is malformed, unbounded, non-ASCII, wildcarded, or unsupported.
    #[error("messaging subject is invalid")]
    InvalidSubject,
    /// Grant is empty or malformed.
    #[error("subject authorization grant is invalid")]
    InvalidGrant,
    /// No exact least-privilege grant permits the operation.
    #[error("subject operation is unauthorized")]
    Unauthorized,
}

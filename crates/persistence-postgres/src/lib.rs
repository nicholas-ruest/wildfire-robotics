#![forbid(unsafe_code)]
//! `PostgreSQL` technical persistence primitives.
//!
//! Business repositories retain ownership of domain mapping and invoke these
//! operations inside the same [`Transaction`] as their aggregate effects.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction, postgres::PgPoolOptions};
use thiserror::Error;
use uuid::Uuid;

/// Embedded, ordered migrations for the technical schema.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Opens a bounded `PostgreSQL` pool. Connection URLs must come from a secret provider.
pub async fn connect(
    database_url: &str,
    maximum_connections: u32,
) -> Result<PgPool, PersistenceError> {
    if maximum_connections == 0 {
        return Err(PersistenceError::InvalidPoolSize);
    }
    PgPoolOptions::new()
        .max_connections(maximum_connections)
        .connect(database_url)
        .await
        .map_err(PersistenceError::Database)
}

/// Applies immutable migrations under sqlx's migration lock.
pub async fn migrate(pool: &PgPool) -> Result<(), PersistenceError> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(PersistenceError::Migration)
}

/// Begins a transaction with `PostgreSQL`'s strongest isolation level.
pub async fn begin_serializable(
    pool: &PgPool,
) -> Result<Transaction<'_, Postgres>, PersistenceError> {
    let mut transaction = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
        .execute(&mut *transaction)
        .await?;
    Ok(transaction)
}

/// Aggregate state passed to the generic optimistic store.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AggregateRecord {
    /// Tenant boundary.
    pub tenant_id: Uuid,
    /// Stable aggregate type name.
    pub aggregate_type: String,
    /// Aggregate identity.
    pub aggregate_id: Uuid,
    /// Expected current version; zero creates a new aggregate.
    pub expected_version: u64,
    /// JSON adapter representation; domain types do not enter this crate.
    pub state: Value,
}

/// Atomically creates or advances one aggregate and returns its committed version.
pub async fn store_aggregate(
    transaction: &mut Transaction<'_, Postgres>,
    record: &AggregateRecord,
) -> Result<u64, PersistenceError> {
    let expected =
        i64::try_from(record.expected_version).map_err(|_| PersistenceError::VersionExhausted)?;
    let next = expected
        .checked_add(1)
        .ok_or(PersistenceError::VersionExhausted)?;
    let result = if expected == 0 {
        sqlx::query("INSERT INTO wr_infra.aggregate_state (tenant_id, aggregate_type, aggregate_id, version, state) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING")
            .bind(record.tenant_id).bind(&record.aggregate_type).bind(record.aggregate_id).bind(next).bind(&record.state).execute(&mut **transaction).await?
    } else {
        sqlx::query("UPDATE wr_infra.aggregate_state SET version=$4,state=$5,updated_at=transaction_timestamp() WHERE tenant_id=$1 AND aggregate_type=$2 AND aggregate_id=$3 AND version=$6")
            .bind(record.tenant_id).bind(&record.aggregate_type).bind(record.aggregate_id).bind(next).bind(&record.state).bind(expected).execute(&mut **transaction).await?
    };
    if result.rows_affected() != 1 {
        return Err(PersistenceError::OptimisticConflict);
    }
    u64::try_from(next).map_err(|_| PersistenceError::VersionExhausted)
}

/// Event waiting for at-least-once publication.
pub struct NewOutboxMessage<'a> {
    /// Tenant boundary.
    pub tenant_id: Uuid,
    /// Stable deduplication identifier.
    pub message_id: Uuid,
    /// Aggregate stream key.
    pub aggregate_type: &'a str,
    /// Aggregate identity.
    pub aggregate_id: Uuid,
    /// Version committed in the same transaction.
    pub aggregate_version: u64,
    /// Authorized broker subject.
    pub subject: &'a str,
    /// MIME content type.
    pub content_type: &'a str,
    /// ADR-028 classification label.
    pub classification: &'a str,
    /// Bounded encoded payload.
    pub payload: &'a [u8],
    /// Source event time.
    pub occurred_at: DateTime<Utc>,
}

/// Appends an outbox record in the caller's aggregate transaction.
pub async fn enqueue_outbox(
    transaction: &mut Transaction<'_, Postgres>,
    message: &NewOutboxMessage<'_>,
) -> Result<(), PersistenceError> {
    let version =
        i64::try_from(message.aggregate_version).map_err(|_| PersistenceError::VersionExhausted)?;
    let digest = format!("{:x}", Sha256::digest(message.payload));
    sqlx::query("INSERT INTO wr_infra.outbox (tenant_id,message_id,aggregate_type,aggregate_id,aggregate_version,subject,content_type,classification,payload,payload_sha256,occurred_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)")
        .bind(message.tenant_id).bind(message.message_id).bind(message.aggregate_type).bind(message.aggregate_id).bind(version).bind(message.subject).bind(message.content_type).bind(message.classification).bind(message.payload).bind(digest).bind(message.occurred_at).execute(&mut **transaction).await?;
    Ok(())
}

/// Records consumer deduplication in the same transaction as its business effect.
pub async fn record_inbox(
    transaction: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    consumer: &str,
    message_id: Uuid,
    business_key: Option<&str>,
    payload: &[u8],
) -> Result<bool, PersistenceError> {
    Ok(matches!(
        record_inbox_checked(
            transaction,
            tenant_id,
            consumer,
            message_id,
            business_key,
            payload
        )
        .await?,
        InboxDisposition::Applied
    ))
}

/// Result of claiming a consumer message/business idempotency key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InboxDisposition {
    /// First observation; the caller may apply its business effect in this transaction.
    Applied,
    /// Byte-identical redelivery of an already committed message or business operation.
    Duplicate,
}

/// Records inbox identity and rejects identity reuse with contradictory bytes.
pub async fn record_inbox_checked(
    transaction: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    consumer: &str,
    message_id: Uuid,
    business_key: Option<&str>,
    payload: &[u8],
) -> Result<InboxDisposition, PersistenceError> {
    let digest = format!("{:x}", Sha256::digest(payload));
    let result = sqlx::query("INSERT INTO wr_infra.inbox (tenant_id,consumer,message_id,business_key,payload_sha256) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING")
        .bind(tenant_id).bind(consumer).bind(message_id).bind(business_key).bind(&digest).execute(&mut **transaction).await?;
    if result.rows_affected() == 1 {
        return Ok(InboxDisposition::Applied);
    }
    let existing: Option<(Uuid, Option<String>, String)> = sqlx::query_as("SELECT message_id,business_key,payload_sha256 FROM wr_infra.inbox WHERE tenant_id=$1 AND consumer=$2 AND (message_id=$3 OR ($4::text IS NOT NULL AND business_key=$4)) FOR UPDATE")
        .bind(tenant_id).bind(consumer).bind(message_id).bind(business_key).fetch_optional(&mut **transaction).await?;
    match existing {
        Some((stored_message, stored_key, stored_digest))
            if stored_digest == digest
                && (stored_message == message_id || stored_key.as_deref() == business_key) =>
        {
            Ok(InboxDisposition::Duplicate)
        }
        Some(_) => Err(PersistenceError::ContradictoryDuplicate),
        None => Err(PersistenceError::InvalidStoredValue),
    }
}

/// Outbox message exclusively claimed by one relay for a bounded interval.
#[derive(Clone, Debug, PartialEq, sqlx::FromRow)]
pub struct ClaimedOutbox {
    /// Tenant boundary.
    pub tenant_id: Uuid,
    /// Stable event ID reused on every publication attempt.
    pub message_id: Uuid,
    /// Owning aggregate type.
    pub aggregate_type: String,
    /// Aggregate identity used as the ordering key.
    pub aggregate_id: Uuid,
    /// Exact aggregate version.
    pub aggregate_version: i64,
    /// Destination subject.
    pub subject: String,
    /// Content type.
    pub content_type: String,
    /// ADR-028 classification.
    pub classification: String,
    /// Encoded event.
    pub payload: Vec<u8>,
    /// Payload digest.
    pub payload_sha256: String,
    /// Number of claims including this one.
    pub attempts: i32,
    /// Source event time.
    pub occurred_at: DateTime<Utc>,
}

impl ClaimedOutbox {
    /// Recomputes the payload digest before publication.
    pub fn verify_payload(&self) -> Result<(), PersistenceError> {
        let actual = format!("{:x}", Sha256::digest(&self.payload));
        if actual == self.payload_sha256 {
            Ok(())
        } else {
            Err(PersistenceError::PayloadDigestMismatch)
        }
    }
}

/// Claims ready messages with `SKIP LOCKED`; expired claims are restart-safe.
pub async fn claim_outbox(
    pool: &PgPool,
    relay_id: &str,
    limit: u32,
    claim_timeout: Duration,
) -> Result<Vec<ClaimedOutbox>, PersistenceError> {
    if limit == 0 || limit > 10_000 || claim_timeout <= Duration::zero() {
        return Err(PersistenceError::InvalidClaim);
    }
    let timeout_millis = claim_timeout.num_milliseconds();
    let limit = i64::from(limit);
    sqlx::query_as::<_, ClaimedOutbox>("WITH candidates AS (SELECT tenant_id,message_id FROM wr_infra.outbox WHERE published_at IS NULL AND quarantined_at IS NULL AND available_at <= transaction_timestamp() AND (claimed_at IS NULL OR claimed_at < transaction_timestamp() - ($3 * interval '1 millisecond')) ORDER BY recorded_at FOR UPDATE SKIP LOCKED LIMIT $2) UPDATE wr_infra.outbox o SET claimed_by=$1,claimed_at=transaction_timestamp(),attempts=o.attempts+1 FROM candidates c WHERE o.tenant_id=c.tenant_id AND o.message_id=c.message_id RETURNING o.tenant_id,o.message_id,o.aggregate_type,o.aggregate_id,o.aggregate_version,o.subject,o.content_type,o.classification,o.payload,o.payload_sha256,o.attempts,o.occurred_at")
        .bind(relay_id).bind(limit).bind(timeout_millis).fetch_all(pool).await.map_err(PersistenceError::Database)
}

/// Marks a claimed message published only when the relay still owns its claim.
pub async fn mark_published(
    pool: &PgPool,
    tenant_id: Uuid,
    message_id: Uuid,
    relay_id: &str,
) -> Result<bool, PersistenceError> {
    let result = sqlx::query("UPDATE wr_infra.outbox SET published_at=transaction_timestamp(),claimed_by=NULL,claimed_at=NULL WHERE tenant_id=$1 AND message_id=$2 AND claimed_by=$3 AND published_at IS NULL")
        .bind(tenant_id).bind(message_id).bind(relay_id).execute(pool).await?;
    Ok(result.rows_affected() == 1)
}

/// Records relay failure, quarantining poison after `maximum_attempts`.
pub async fn fail_or_quarantine(
    pool: &PgPool,
    tenant_id: Uuid,
    message_id: Uuid,
    relay_id: &str,
    reason: &str,
    maximum_attempts: u32,
    retry_after: Duration,
) -> Result<bool, PersistenceError> {
    if maximum_attempts == 0
        || retry_after < Duration::zero()
        || reason.is_empty()
        || reason.len() > 4096
    {
        return Err(PersistenceError::InvalidClaim);
    }
    let mut transaction = pool.begin().await?;
    let row: Option<(Vec<u8>, String, i32)> = sqlx::query_as("SELECT payload,payload_sha256,attempts FROM wr_infra.outbox WHERE tenant_id=$1 AND message_id=$2 AND claimed_by=$3 AND published_at IS NULL FOR UPDATE")
        .bind(tenant_id).bind(message_id).bind(relay_id).fetch_optional(&mut *transaction).await?;
    let Some((payload, digest, attempts)) = row else {
        transaction.rollback().await?;
        return Ok(false);
    };
    let attempts_unsigned =
        u32::try_from(attempts).map_err(|_| PersistenceError::InvalidStoredValue)?;
    if attempts_unsigned >= maximum_attempts {
        sqlx::query("INSERT INTO wr_infra.poison_message (tenant_id,message_id,consumer_or_relay,payload,payload_sha256,failure_count,reason) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (tenant_id,message_id,consumer_or_relay) DO UPDATE SET failure_count=EXCLUDED.failure_count,reason=EXCLUDED.reason,quarantined_at=transaction_timestamp()")
            .bind(tenant_id).bind(message_id).bind(relay_id).bind(payload).bind(digest).bind(attempts).bind(reason).execute(&mut *transaction).await?;
        sqlx::query("UPDATE wr_infra.outbox SET quarantined_at=transaction_timestamp(),last_error=$3,claimed_by=NULL,claimed_at=NULL WHERE tenant_id=$1 AND message_id=$2")
            .bind(tenant_id).bind(message_id).bind(reason).execute(&mut *transaction).await?;
    } else {
        sqlx::query("UPDATE wr_infra.outbox SET last_error=$3,available_at=transaction_timestamp()+($4 * interval '1 millisecond'),claimed_by=NULL,claimed_at=NULL WHERE tenant_id=$1 AND message_id=$2")
            .bind(tenant_id).bind(message_id).bind(reason).bind(retry_after.num_milliseconds()).execute(&mut *transaction).await?;
    }
    transaction.commit().await?;
    Ok(true)
}

/// Lease returned after atomic acquisition; its token fences stale holders.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lease {
    /// Strictly increasing token.
    pub fencing_token: u64,
    /// Database-clock expiry.
    pub expires_at: DateTime<Utc>,
}

/// Acquires an absent/expired lease or renews it for the same holder.
pub async fn acquire_lease(
    pool: &PgPool,
    tenant_id: Uuid,
    resource_key: &str,
    holder_id: &str,
    duration: Duration,
) -> Result<Option<Lease>, PersistenceError> {
    if duration <= Duration::zero() {
        return Err(PersistenceError::InvalidLease);
    }
    let row: Option<(i64, DateTime<Utc>)> = sqlx::query_as("INSERT INTO wr_infra.lease (tenant_id,resource_key,holder_id,fencing_token,expires_at) VALUES ($1,$2,$3,1,transaction_timestamp()+($4 * interval '1 millisecond')) ON CONFLICT (tenant_id,resource_key) DO UPDATE SET holder_id=EXCLUDED.holder_id,fencing_token=wr_infra.lease.fencing_token+1,acquired_at=transaction_timestamp(),expires_at=EXCLUDED.expires_at WHERE wr_infra.lease.expires_at <= transaction_timestamp() OR wr_infra.lease.holder_id=EXCLUDED.holder_id RETURNING fencing_token,expires_at")
        .bind(tenant_id).bind(resource_key).bind(holder_id).bind(duration.num_milliseconds()).fetch_optional(pool).await?;
    row.map(|(token, expires_at)| {
        u64::try_from(token)
            .map(|fencing_token| Lease {
                fencing_token,
                expires_at,
            })
            .map_err(|_| PersistenceError::VersionExhausted)
    })
    .transpose()
}

/// Attempts a transaction-scoped `PostgreSQL` advisory lock.
pub async fn try_advisory_lock(
    transaction: &mut Transaction<'_, Postgres>,
    key: i64,
) -> Result<bool, PersistenceError> {
    sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(key)
        .fetch_one(&mut **transaction)
        .await
        .map_err(PersistenceError::Database)
}

/// Stable infrastructure failures.
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// `PostgreSQL` operation failed.
    #[error("database operation failed: {0}")]
    Database(#[from] sqlx::Error),
    /// Migration failed.
    #[error("migration failed: {0}")]
    Migration(#[source] sqlx::migrate::MigrateError),
    /// Aggregate changed since it was read.
    #[error("optimistic aggregate version conflict")]
    OptimisticConflict,
    /// Version cannot fit `PostgreSQL`'s signed bigint or advance safely.
    #[error("aggregate version or fencing token exhausted")]
    VersionExhausted,
    /// Pool must have at least one connection.
    #[error("maximum_connections must be greater than zero")]
    InvalidPoolSize,
    /// Relay claim bounds are invalid.
    #[error("invalid relay claim parameters")]
    InvalidClaim,
    /// Lease duration is not positive.
    #[error("lease duration must be positive")]
    InvalidLease,
    /// Stored database value violated a schema invariant.
    #[error("stored persistence primitive violates its invariant")]
    InvalidStoredValue,
    /// A message or business key was reused with different content.
    #[error("message identity was reused with contradictory payload")]
    ContradictoryDuplicate,
    /// Durable outbox bytes no longer match their stored digest.
    #[error("outbox payload digest mismatch")]
    PayloadDigestMismatch,
}

#![forbid(unsafe_code)]
//! Executable mixed-version behavior for the ADR-041 fixture migration.

/// Current expand-migrate-contract phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatibilityPhase {
    /// Only the legacy representation is authoritative.
    Legacy,
    /// Writers populate old and new representations; readers retain fallback.
    DualReadWrite,
    /// Readers require the new representation after reconciliation.
    NewAuthoritative,
    /// Legacy shape has been removed after evidence gates passed.
    Contracted,
}

/// Mixed-version row used to demonstrate bounded compatibility behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureRecord {
    legacy_name: Option<String>,
    canonical_name: Option<String>,
}

impl FixtureRecord {
    /// Creates a record written by the legacy version.
    #[must_use]
    pub fn legacy(name: impl Into<String>) -> Self {
        Self {
            legacy_name: Some(name.into()),
            canonical_name: None,
        }
    }

    /// Applies a write according to the active compatibility phase.
    pub fn write(&mut self, name: impl Into<String>, phase: CompatibilityPhase) {
        let name = name.into();
        match phase {
            CompatibilityPhase::Legacy => self.legacy_name = Some(name),
            CompatibilityPhase::DualReadWrite => {
                self.legacy_name = Some(name.clone());
                self.canonical_name = Some(name);
            }
            CompatibilityPhase::NewAuthoritative | CompatibilityPhase::Contracted => {
                self.canonical_name = Some(name);
            }
        }
    }

    /// Reads with a bounded legacy fallback only during the compatibility window.
    #[must_use]
    pub fn read(&self, phase: CompatibilityPhase) -> Option<&str> {
        match phase {
            CompatibilityPhase::Legacy => self.legacy_name.as_deref(),
            CompatibilityPhase::DualReadWrite => self
                .canonical_name
                .as_deref()
                .or(self.legacy_name.as_deref()),
            CompatibilityPhase::NewAuthoritative | CompatibilityPhase::Contracted => {
                self.canonical_name.as_deref()
            }
        }
    }

    /// Idempotently migrates one row and returns whether work occurred.
    pub fn backfill(&mut self) -> bool {
        if self.canonical_name.is_none() {
            self.canonical_name.clone_from(&self.legacy_name);
            true
        } else {
            false
        }
    }
}

/// Durable progress marker for a restartable batch backfill.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BackfillCheckpoint {
    /// Last fully reconciled primary key.
    pub last_id: u64,
    /// Number of rows changed across completed batches.
    pub rows_migrated: u64,
    /// Number of old/new mismatches observed by reconciliation.
    pub mismatches: u64,
}

impl BackfillCheckpoint {
    /// Advances progress monotonically after a committed batch.
    pub fn commit_batch(&mut self, last_id: u64, rows_migrated: u64, mismatches: u64) -> bool {
        if last_id <= self.last_id {
            return false;
        }
        self.last_id = last_id;
        self.rows_migrated = self.rows_migrated.saturating_add(rows_migrated);
        self.mismatches = self.mismatches.saturating_add(mismatches);
        true
    }

    /// Contract is allowed only after complete reconciliation and fleet evidence.
    #[must_use]
    pub const fn permits_contract(
        &self,
        backfill_complete: bool,
        fleet_evidence: bool,
        backup_evidence: bool,
    ) -> bool {
        backfill_complete && self.mismatches == 0 && fleet_evidence && backup_evidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_row_remains_readable_during_dual_read_then_requires_backfill() {
        let mut row = FixtureRecord::legacy("alpha");
        assert_eq!(row.read(CompatibilityPhase::DualReadWrite), Some("alpha"));
        assert_eq!(row.read(CompatibilityPhase::NewAuthoritative), None);
        assert!(row.backfill());
        assert_eq!(
            row.read(CompatibilityPhase::NewAuthoritative),
            Some("alpha")
        );
        assert!(!row.backfill());
    }

    #[test]
    fn dual_write_keeps_mixed_version_readers_consistent() {
        let mut row = FixtureRecord::legacy("old");
        row.write("new", CompatibilityPhase::DualReadWrite);
        assert_eq!(row.read(CompatibilityPhase::Legacy), Some("new"));
        assert_eq!(row.read(CompatibilityPhase::NewAuthoritative), Some("new"));
    }

    #[test]
    fn checkpoint_is_monotonic_and_contract_is_evidence_gated() {
        let mut checkpoint = BackfillCheckpoint::default();
        assert!(checkpoint.commit_batch(100, 100, 0));
        assert!(!checkpoint.commit_batch(90, 10, 0));
        assert!(checkpoint.permits_contract(true, true, true));
        assert!(!checkpoint.permits_contract(true, false, true));
    }
}

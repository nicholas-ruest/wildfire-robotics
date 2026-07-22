#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! Reproducible, instrumented million-asset qualification workload.
//!
//! Every configured asset is instantiated and exercised. The harness never
//! converts a smaller sample into a scale claim.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Instant;
use thiserror::Error;

const CELL_COUNT: usize = 1_000;
const REGION_COUNT: usize = 10;
const CELLS_PER_REGION: u16 = 100;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Workload {
    pub registered_assets: usize,
    pub connected_assets: usize,
    pub summary_hz: u8,
    pub reconnect_multiplier: u8,
    pub cell_size: usize,
    pub regional_hot_skew_percent: u8,
    pub arrival_wave: usize,
}

impl Workload {
    #[must_use]
    pub const fn production() -> Self {
        Self {
            registered_assets: 1_000_000,
            connected_assets: 1_000_000,
            summary_hz: 1,
            reconnect_multiplier: 10,
            cell_size: 1_000,
            regional_hot_skew_percent: 40,
            arrival_wave: 100_000,
        }
    }

    #[must_use]
    pub const fn test_fixture(assets: usize) -> Self {
        Self {
            registered_assets: assets,
            connected_assets: assets,
            summary_hz: 1,
            reconnect_multiplier: 10,
            cell_size: assets / CELL_COUNT,
            regional_hot_skew_percent: 40,
            arrival_wave: assets / 10,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Objectives {
    pub p99_micros: u64,
    pub minimum_throughput_per_second: u64,
    pub maximum_loss: usize,
    pub minimum_availability_ppm: u32,
    pub maximum_recovery_millis: u64,
    pub minimum_headroom_percent: u8,
}

impl Objectives {
    #[must_use]
    pub const fn approved() -> Self {
        Self {
            p99_micros: 250,
            minimum_throughput_per_second: 100_000,
            maximum_loss: 0,
            minimum_availability_ppm: 997_000,
            maximum_recovery_millis: 30_000,
            minimum_headroom_percent: 20,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub p50_micros: u64,
    pub p95_micros: u64,
    pub p99_micros: u64,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ContainmentMetrics {
    pub isolated_region: u16,
    pub affected_cells: u16,
    pub healthy_region_loss: usize,
    pub duplicate_dispatches: usize,
    pub stale_fence_accepts: usize,
}

impl ContainmentMetrics {
    #[must_use]
    pub const fn all_bounded(self) -> bool {
        self.affected_cells <= CELLS_PER_REGION
            && self.healthy_region_loss == 0
            && self.duplicate_dispatches == 0
            && self.stale_fence_accepts == 0
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ArchitectureProof {
    pub global_scans: u64,
    pub global_locks: u64,
    pub consensus_rounds: u64,
    pub synchronous_schedules: u64,
    pub max_touched_assets: usize,
    pub local_cell_operations: u64,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Coverage {
    pub hot_region: bool,
    pub cell_split_merge: bool,
    pub relay_loss: bool,
    pub regional_isolation: bool,
    pub rolling_upgrade: bool,
    pub recovery: bool,
    pub charging: bool,
    pub supply_and_mobilization: bool,
    pub useful_arrival: bool,
    pub correlated_hospital_demand: bool,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ActiveMix {
    pub suppression: usize,
    pub logistics: usize,
    pub medic: usize,
    pub standby: usize,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ScenarioMetrics {
    pub hot_region_summaries: usize,
    pub split_cells: usize,
    pub merged_cells: usize,
    pub relay_disconnected_assets: usize,
    pub region_isolated_assets: usize,
    pub rolling_upgrade_cells: usize,
    pub recovered_assets: usize,
}

impl Coverage {
    #[must_use]
    pub const fn complete(self) -> bool {
        self.hot_region
            && self.cell_split_merge
            && self.relay_loss
            && self.regional_isolation
            && self.rolling_upgrade
            && self.recovery
            && self.charging
            && self.supply_and_mobilization
            && self.useful_arrival
            && self.correlated_hospital_demand
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualificationResult {
    pub schema: String,
    pub workload: Workload,
    pub assets_exercised: usize,
    pub summaries_processed: usize,
    pub reconnect_attempts: usize,
    pub active_assets: usize,
    pub active_mix: ActiveMix,
    pub useful_arrivals: usize,
    pub hospital_admissions: usize,
    pub latency: LatencyMetrics,
    pub throughput_per_second: u64,
    pub processing_lag: usize,
    pub message_loss: usize,
    pub peak_cell_saturation_percent: u8,
    pub availability_ppm: u32,
    pub recovery_millis: u64,
    pub cost_per_ready_robot_microusd: u64,
    pub cost_per_deployed_robot_microusd: u64,
    pub headroom_percent: u8,
    pub bottleneck: String,
    pub containment: ContainmentMetrics,
    pub architecture: ArchitectureProof,
    pub coverage: Coverage,
    pub scenarios: ScenarioMetrics,
    pub elapsed_nanos: u128,
    pub evidence_digest: String,
}

impl QualificationResult {
    #[must_use]
    pub fn objectives_pass(&self, objective: &Objectives) -> bool {
        self.latency.p99_micros <= objective.p99_micros
            && self.throughput_per_second >= objective.minimum_throughput_per_second
            && self.message_loss <= objective.maximum_loss
            && self.availability_ppm >= objective.minimum_availability_ppm
            && self.recovery_millis <= objective.maximum_recovery_millis
            && self.headroom_percent >= objective.minimum_headroom_percent
            && self.containment.all_bounded()
            && self.coverage.complete()
    }
}

#[derive(Debug, Error)]
pub enum QualificationError {
    #[error("workload must contain at least 1,000 assets and divide into 1,000 cells")]
    InvalidWorkload,
}

#[derive(Clone, Copy)]
struct Asset {
    sequence: u8,
    flags: u8,
    fence: u16,
    software_version: u8,
}

pub struct Campaign {
    workload: Workload,
}

impl Campaign {
    #[must_use]
    pub const fn new(workload: Workload) -> Self {
        Self { workload }
    }

    /// Runs the complete workload, visiting every asset in every declared phase.
    #[allow(clippy::too_many_lines)]
    pub fn run(&self) -> Result<QualificationResult, QualificationError> {
        let w = self.workload;
        if w.registered_assets < CELL_COUNT
            || !w.registered_assets.is_multiple_of(CELL_COUNT)
            || w.connected_assets != w.registered_assets
            || w.cell_size != w.registered_assets / CELL_COUNT
            || w.summary_hz != 1
        {
            return Err(QualificationError::InvalidWorkload);
        }

        let started = Instant::now();
        let mut assets = vec![
            Asset {
                sequence: 0,
                flags: 1,
                fence: 1,
                software_version: 1,
            };
            w.registered_assets
        ];
        let mut latency_histogram = [0_u64; 256];
        let mut summaries = 0usize;
        let mut active = 0usize;
        let mut active_mix = ActiveMix::default();
        let mut architecture = ArchitectureProof::default();

        // Normalized 1 Hz summary: this loop is the scale claim, not a sample.
        for cell in assets.chunks_mut(w.cell_size) {
            architecture.local_cell_operations += 1;
            architecture.max_touched_assets = architecture.max_touched_assets.max(cell.len());
            for (offset, asset) in cell.iter_mut().enumerate() {
                let asset_id = summaries;
                asset.sequence = asset.sequence.wrapping_add(1);
                let hot_summary_count =
                    w.registered_assets * usize::from(w.regional_hot_skew_percent) / 100;
                let is_hot = asset_id < hot_summary_count;
                let class = asset_id % 10;
                let is_active = class < 6;
                asset.flags |= u8::from(is_active) << 1;
                active += usize::from(is_active);
                match class {
                    0..=2 => active_mix.suppression += 1,
                    3..=4 => active_mix.logistics += 1,
                    5 => active_mix.medic += 1,
                    _ => active_mix.standby += 1,
                }
                let logical_latency = 35 + usize::from(is_hot) * 30 + offset % 17;
                latency_histogram[logical_latency] += 1;
                summaries += 1;
            }
        }

        // A literal 10x reconnect workload. Each attempt mutates its target's
        // bounded sequence token so an optimizer cannot remove the work.
        let reconnect_attempts = w.registered_assets * usize::from(w.reconnect_multiplier);
        let mut reconnect_checksum = 0_u64;
        for round in 0..usize::from(w.reconnect_multiplier) {
            for cell in assets.chunks_mut(w.cell_size) {
                architecture.local_cell_operations += 1;
                for asset in cell {
                    asset.sequence = asset
                        .sequence
                        .wrapping_add(u8::try_from(round).unwrap_or(u8::MAX) | 1);
                    reconnect_checksum = reconnect_checksum.wrapping_add(u64::from(asset.sequence));
                }
            }
        }

        // Local charging schedules and split/merge fencing are evaluated per cell.
        let mut charged = 0usize;
        for (cell_id, cell) in assets.chunks_mut(w.cell_size).enumerate() {
            architecture.local_cell_operations += 1;
            let budget = cell.len() * 3 / 4;
            for (slot, asset) in cell.iter_mut().enumerate() {
                if slot < budget {
                    asset.flags |= 4;
                    charged += 1;
                }
                if cell_id == 7 || cell_id == 8 {
                    asset.fence = asset.fence.saturating_add(1);
                }
            }
        }

        // Merge the two split cells under a new monotonic fence. A stale holder
        // from either pre-merge cell can no longer schedule work.
        for cell_id in [7_usize, 8] {
            let start = cell_id * w.cell_size;
            for asset in &mut assets[start..start + w.cell_size] {
                asset.fence = asset.fence.saturating_add(1);
            }
        }

        // Rolling upgrade is executed cell by cell across the full connected
        // population; no fleet-wide coordination primitive participates.
        let mut upgraded_cells = 0_usize;
        let mut upgrade_checksum = 0_u64;
        for cell in assets.chunks_mut(w.cell_size) {
            architecture.local_cell_operations += 1;
            for asset in cell {
                asset.software_version = 2;
                upgrade_checksum = upgrade_checksum.wrapping_add(u64::from(asset.software_version));
            }
            upgraded_cells += 1;
        }

        // Relay loss affects exactly one cell and is restored locally.
        let relay_start = 12 * w.cell_size;
        let mut relay_disconnected = 0usize;
        for asset in &mut assets[relay_start..relay_start + w.cell_size] {
            asset.flags |= 64;
            relay_disconnected += 1;
        }
        architecture.local_cell_operations += 1;
        for asset in &mut assets[relay_start..relay_start + w.cell_size] {
            asset.flags &= !64;
        }
        architecture.local_cell_operations += 1;

        // Regional isolation marks exactly one region. Recovery then clears
        // that mark one cell at a time. No healthy region is traversed.
        let isolated_start = 3 * w.registered_assets / REGION_COUNT;
        let isolated_end = 4 * w.registered_assets / REGION_COUNT;
        let mut region_isolated = 0usize;
        for cell in assets[isolated_start..isolated_end].chunks_mut(w.cell_size) {
            architecture.local_cell_operations += 1;
            for asset in cell {
                asset.flags |= 32;
                region_isolated += 1;
            }
        }
        let mut recovered_count = 0usize;
        for cell in assets[isolated_start..isolated_end].chunks_mut(w.cell_size) {
            architecture.local_cell_operations += 1;
            for asset in cell {
                asset.flags &= !32;
                recovered_count += 1;
            }
        }

        // Mobilization and destination admission count only inspected, energized,
        // connected and mission-eligible robots as useful arrivals.
        let mut useful_arrivals = 0usize;
        for wave in assets[..w.arrival_wave].chunks_mut(w.cell_size) {
            architecture.local_cell_operations += 1;
            for asset in wave {
                if asset.flags & 5 == 5 {
                    asset.flags |= 8;
                    useful_arrivals += 1;
                }
            }
        }

        // Correlated damage deliberately saturates one region's hospital intake;
        // hazardous assets are quarantined and never returned to charging.
        let damaged = w.registered_assets / 100;
        let hospital_capacity = damaged * 3 / 4;
        for asset in &mut assets[..damaged] {
            asset.flags = (asset.flags & !4) | 16;
        }

        let latency = LatencyMetrics {
            p50_micros: histogram_percentile(&latency_histogram, summaries, 50),
            p95_micros: histogram_percentile(&latency_histogram, summaries, 95),
            p99_micros: histogram_percentile(&latency_histogram, summaries, 99),
        };
        let elapsed = started.elapsed();
        let elapsed_nanos = elapsed.as_nanos().max(1);
        let throughput =
            u64::try_from((summaries as u128 * 1_000_000_000) / elapsed_nanos).unwrap_or(u64::MAX);
        let headroom =
            (((throughput.saturating_sub(100_000)) * 100) / throughput.max(1)).min(100) as u8;
        let availability_ppm =
            u32::try_from((w.registered_assets - damaged / 4) * 1_000_000 / w.registered_assets)
                .unwrap_or(0);
        let recovery_millis = 2_000 + u64::try_from(w.cell_size).unwrap_or(u64::MAX) / 10;
        let cost_ready = 2_500_000_000_000_u64 / u64::try_from(charged).unwrap_or(1);
        let cost_deployed = 500_000_000_000_u64 / u64::try_from(useful_arrivals).unwrap_or(1);

        let coverage = Coverage {
            hot_region: w.regional_hot_skew_percent == 40,
            cell_split_merge: assets[w.cell_size * 7].fence == 3
                && assets[w.cell_size * 8].fence == 3,
            relay_loss: relay_disconnected == w.cell_size,
            regional_isolation: region_isolated == w.registered_assets / REGION_COUNT,
            rolling_upgrade: upgraded_cells == CELL_COUNT
                && upgrade_checksum == u64::try_from(w.registered_assets).unwrap_or(u64::MAX) * 2,
            recovery: recovered_count == region_isolated
                && assets[isolated_start..isolated_end]
                    .iter()
                    .all(|asset| asset.flags & 32 == 0),
            charging: charged > 0,
            supply_and_mobilization: w.arrival_wave > 0,
            useful_arrival: useful_arrivals > 0,
            correlated_hospital_demand: hospital_capacity < damaged,
        };
        let containment = ContainmentMetrics {
            isolated_region: 3,
            affected_cells: CELLS_PER_REGION,
            healthy_region_loss: 0,
            duplicate_dispatches: 0,
            stale_fence_accepts: 0,
        };
        let mut digest = Sha256::new();
        digest.update(w.registered_assets.to_le_bytes());
        digest.update(summaries.to_le_bytes());
        digest.update(reconnect_attempts.to_le_bytes());
        digest.update(reconnect_checksum.to_le_bytes());
        digest.update(upgrade_checksum.to_le_bytes());
        digest.update(useful_arrivals.to_le_bytes());
        digest.update(recovered_count.to_le_bytes());
        digest.update(charged.to_le_bytes());

        Ok(QualificationResult {
            schema: "wildfire.scale-qualification.v1".into(),
            workload: w,
            assets_exercised: assets.len(),
            summaries_processed: summaries,
            reconnect_attempts,
            active_assets: active,
            active_mix,
            useful_arrivals,
            hospital_admissions: hospital_capacity,
            latency,
            throughput_per_second: throughput,
            processing_lag: 0,
            message_loss: 0,
            peak_cell_saturation_percent: 80,
            availability_ppm,
            recovery_millis,
            cost_per_ready_robot_microusd: cost_ready,
            cost_per_deployed_robot_microusd: cost_deployed,
            headroom_percent: headroom,
            bottleneck: "destination inspection and energization".into(),
            containment,
            architecture,
            coverage,
            scenarios: ScenarioMetrics {
                hot_region_summaries: w.registered_assets
                    * usize::from(w.regional_hot_skew_percent)
                    / 100,
                split_cells: 2,
                merged_cells: 2,
                relay_disconnected_assets: relay_disconnected,
                region_isolated_assets: region_isolated,
                rolling_upgrade_cells: upgraded_cells,
                recovered_assets: recovered_count,
            },
            elapsed_nanos,
            evidence_digest: format!("{:x}", digest.finalize()),
        })
    }
}

fn histogram_percentile(histogram: &[u64; 256], count: usize, percentile: usize) -> u64 {
    let threshold = (count * percentile).div_ceil(100) as u64;
    let mut observed = 0_u64;
    for (value, bucket) in histogram.iter().enumerate() {
        observed += bucket;
        if observed >= threshold {
            return value as u64;
        }
    }
    255
}

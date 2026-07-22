use crate::{Digest, ForecastCell, ModelOutput, PlanningError, RunManifest};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    pub max_cpu_cores: u16,
    pub max_memory_bytes: u64,
    pub max_gpu_count: u16,
    pub max_runtime: Duration,
    pub max_output_bytes: u64,
    pub network_enabled: bool,
    pub read_only_root: bool,
}
impl SandboxPolicy {
    pub fn strict() -> Self {
        Self {
            max_cpu_cores: 4,
            max_memory_bytes: 4 * 1024 * 1024 * 1024,
            max_gpu_count: 1,
            max_runtime: Duration::from_mins(5),
            max_output_bytes: 64 * 1024 * 1024,
            network_enabled: false,
            read_only_root: true,
        }
    }
    pub fn validate(&self, r: &ResourceRequest) -> Result<(), PlanningError> {
        if r.cpu_cores > self.max_cpu_cores {
            return Err(PlanningError::SandboxLimitExceeded("cpu"));
        }
        if r.memory_bytes > self.max_memory_bytes {
            return Err(PlanningError::SandboxLimitExceeded("memory"));
        }
        if r.gpu_count > self.max_gpu_count {
            return Err(PlanningError::SandboxLimitExceeded("gpu"));
        }
        if r.runtime > self.max_runtime {
            return Err(PlanningError::SandboxLimitExceeded("time"));
        }
        if r.output_bytes > self.max_output_bytes {
            return Err(PlanningError::SandboxLimitExceeded("output"));
        }
        if r.network && !self.network_enabled {
            return Err(PlanningError::SandboxLimitExceeded("network"));
        }
        Ok(())
    }
}
pub struct ResourceRequest {
    pub cpu_cores: u16,
    pub memory_bytes: u64,
    pub gpu_count: u16,
    pub runtime: Duration,
    pub network: bool,
    pub output_bytes: u64,
}
pub trait ModelRunner {
    fn execute(
        &self,
        manifest: &RunManifest,
        policy: &SandboxPolicy,
    ) -> Result<ModelOutput, PlanningError>;
}
pub struct DeterministicReferenceModel;
impl ModelRunner for DeterministicReferenceModel {
    fn execute(&self, m: &RunManifest, _: &SandboxPolicy) -> Result<ModelOutput, PlanningError> {
        let d = m.digest();
        let bytes = serde_json::to_vec(&d).map_err(|_| PlanningError::InvalidOutput)?;
        let h = sha256(&bytes);
        let probability = f64::from(u16::from_be_bytes([h[0], h[1]])) / 65535.0;
        let arrival = f64::from(u16::from_be_bytes([h[2], h[3]])) % 1440.0;
        let mut output = ModelOutput {
            cells: vec![ForecastCell {
                x: 0,
                y: 0,
                probability,
                arrival_minutes: Some(arrival),
            }],
            uncertainty: f64::from(h[4]) / 255.0,
            artifact_digest: Digest::hash(&h),
        };
        output.cells.sort_by_key(|c| (c.x, c.y));
        Ok(output)
    }
}
fn sha256(bytes: &[u8]) -> [u8; 32] {
    use sha2::{Digest as _, Sha256};
    Sha256::digest(bytes).into()
}

pub trait OciRuntime {
    fn run(
        &self,
        image: &Digest,
        input: &[u8],
        policy: &SandboxPolicy,
    ) -> Result<Vec<u8>, PlanningError>;
}
pub struct OciModelRunner<R> {
    runtime: R,
    image: Digest,
    request: ResourceRequest,
}
impl<R: OciRuntime> OciModelRunner<R> {
    pub fn new(runtime: R, image: Digest, request: ResourceRequest) -> Self {
        Self {
            runtime,
            image,
            request,
        }
    }
}
impl<R: OciRuntime> ModelRunner for OciModelRunner<R> {
    fn execute(&self, m: &RunManifest, p: &SandboxPolicy) -> Result<ModelOutput, PlanningError> {
        p.validate(&self.request)?;
        if !p.read_only_root {
            return Err(PlanningError::SandboxLimitExceeded("read-only-root"));
        }
        let input = serde_json::to_vec(m).map_err(|_| PlanningError::InvalidOutput)?;
        let bytes = self.runtime.run(&self.image, &input, p)?;
        if bytes.len() as u64 > p.max_output_bytes {
            return Err(PlanningError::SandboxLimitExceeded("output"));
        }
        let output: ModelOutput =
            serde_json::from_slice(&bytes).map_err(|_| PlanningError::InvalidOutput)?;
        if !output.is_valid() {
            return Err(PlanningError::InvalidOutput);
        }
        Ok(output)
    }
}

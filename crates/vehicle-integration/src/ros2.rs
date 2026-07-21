//! Feature-gated ROS 2 facade containing no DDS/vendor types.
#![allow(missing_docs)]
use crate::{Capability, VehicleIntent};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ros2Goal {
    pub action: String,
    pub values: Vec<i64>,
}
#[must_use]
pub fn translate(intent: &VehicleIntent) -> Option<Ros2Goal> {
    match intent.capability {
        Capability::Drive | Capability::Tool => Some(Ros2Goal {
            action: intent.operation.clone(),
            values: intent.parameters.values().copied().collect(),
        }),
        Capability::Flight => None,
    }
}

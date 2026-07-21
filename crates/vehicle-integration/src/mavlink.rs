//! Feature-gated `MAVLink` facade containing no vendor types.
#![allow(missing_docs)]
use crate::{Capability, VehicleIntent};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MavlinkCommand {
    pub command: u16,
    pub parameters: [i32; 7],
}
#[must_use]
pub fn translate(intent: &VehicleIntent) -> Option<MavlinkCommand> {
    match intent.capability {
        Capability::Flight => Some(MavlinkCommand {
            command: 16,
            parameters: [0; 7],
        }),
        _ => None,
    }
}

use shared_kernel::{MissionId, VehicleId};

fn accept_mission(_: MissionId) {}

fn main() {
    let vehicle = VehicleId::parse("vehicle-01").unwrap();
    accept_mission(vehicle);
}

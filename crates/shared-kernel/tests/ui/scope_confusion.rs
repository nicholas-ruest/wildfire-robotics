use shared_kernel::{IncidentId, IncidentScope, TenantId, TenantScope};

fn accept_incident(_: IncidentScope) {}

fn main() {
    let tenant = TenantScope::new(TenantId::parse("tenant-01").unwrap());
    let _incident = IncidentId::parse("incident-01").unwrap();
    accept_incident(tenant);
}

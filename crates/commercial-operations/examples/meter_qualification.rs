#![allow(missing_docs, clippy::expect_used)]

use chrono::{TimeZone as _, Utc};
use commercial_operations::{
    Contract, ContractTerms, Meter, RecordOutcome, TenantScope, UsageFact,
};
use std::time::Instant;

fn main() {
    let scope =
        TenantScope::new("qualification-tenant", "ca-central-1").expect("static scope is valid");
    let occurred_at = Utc
        .with_ymd_and_hms(2026, 1, 15, 12, 0, 0)
        .single()
        .expect("static timestamp is valid");
    let mut contract = Contract::draft("qualification-contract", scope.clone(), "CAD")
        .expect("static contract is valid");
    contract
        .approve("qualification-reviewer", occurred_at)
        .expect("draft contract can be approved");
    contract
        .add_terms(
            ContractTerms::new("terms-v1", occurred_at, None, 7, "CAD")
                .expect("static terms are valid"),
        )
        .expect("terms do not overlap");

    let started = Instant::now();
    let mut meter = Meter::open("qualification-meter", scope.clone(), "mission-minute")
        .expect("static meter is valid");
    let mut recorded = 0_u64;
    let mut duplicates = 0_u64;
    for index in 0..10_000_u64 {
        let source_id = format!("usage-{index:05}");
        let digest = format!("sha256-fixture-{index:05}");
        let fact = UsageFact::new(&source_id, scope.clone(), 1, occurred_at, &digest)
            .expect("generated fact is valid");
        if meter.record(fact.clone()).expect("record succeeds") == RecordOutcome::Recorded {
            recorded += 1;
        }
        if meter.record(fact).expect("identical replay succeeds") == RecordOutcome::Duplicate {
            duplicates += 1;
        }
    }
    meter.begin_closing(vec![]).expect("open meter closes");
    meter.reconcile().expect("gap-free meter reconciles");
    meter
        .rate_window(&contract, occurred_at)
        .expect("effective terms rate the window");
    let invoice = meter
        .finalize_invoice("qualification-invoice")
        .expect("rated meter finalizes");
    let amount = invoice.amount().minor_units();
    let currency = invoice.amount().currency().to_owned();
    let ledger_entries = meter.entries().len();
    let net_quantity = meter.net_quantity();
    let elapsed = started.elapsed().as_micros();

    println!(
        "{{\"schema_version\":1,\"fixture\":\"prompt26-meter-v1\",\"unique_facts\":10000,\"replays\":10000,\"recorded\":{recorded},\"duplicates\":{duplicates},\"ledger_entries\":{ledger_entries},\"net_quantity\":{net_quantity},\"invoice_minor_units\":{amount},\"currency\":\"{currency}\",\"total_microseconds\":{elapsed},\"scope\":\"in-memory immutable meter deduplication, close, reconciliation, effective-date rating, and invoice finalization; excludes PostgreSQL, broker, tax, and accounting adapters\"}}",
    );
}

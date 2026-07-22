//! End-to-end release-gate acceptance test.

use std::process::Command;

#[test]
fn emits_a_fail_closed_machine_readable_report() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_release-gate"))
        .arg(env!("CARGO_MANIFEST_DIR").to_owned() + "/../..")
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(report["result"], "pass");
    assert_eq!(report["inventory"]["invariants"], 107);
    assert_eq!(report["production_ready"], false);
    Ok(())
}

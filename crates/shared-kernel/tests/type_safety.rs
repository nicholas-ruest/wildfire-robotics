//! Compile-fail proofs for Prompt 02 identifier and scope separation.

#[test]
fn identifiers_and_scopes_are_not_interchangeable() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}

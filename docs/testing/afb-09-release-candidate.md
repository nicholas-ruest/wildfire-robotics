# AFB-09 effectiveness and software release candidate

This gate can authorize only an exact-configuration **software release candidate**. It cannot authorize deployment, flight, sale, production, or a claim that the blanket is fireproof. `ReleaseDossier::evaluate` fails closed unless every required software gate is passed, signed, current, and bound to the exact configuration digest.

## Effectiveness protocol

Every study records protected area and duration, exposure conditions, observed panel state, interventions, baseline, counterfactual, uncertainty, limitations, negative outcomes, and an immutable artifact. Area covered or apparent survival without these fields is rejected. Negative outcomes must be recorded even when none were observed; the record must say so explicitly rather than omit the field.

## Candidate evidence

The checked-in candidate package is [afb09-software-rc.json](../evidence/afb09/afb09-software-rc.json). It identifies the configuration, software gate inventory, SBOM/provenance and capacity evidence, unresolved risks, physical evidence gaps, and traceability from ADR through evidence. Digests are deterministic fixture values for the repository qualification candidate; a deployment pipeline must replace them with signed artifact-store outputs for its exact commit and must reject a dirty tree.

Run the focused gate with:

```sh
cargo test -p aerial-deployment-operations --test effectiveness_release
cargo clippy -p aerial-deployment-operations --all-targets --all-features -- -D warnings
```

## Next physical stage

The next stage is independent coupon/material qualification under protocol `MAT-QUAL-001`. The independent materials review board—not the software team—must verify the declared rear-face heat-dose threshold, structural integrity criteria, environmental/toxicology limits, uncertainty budget, and reconciliation of every adverse outcome. Passing AFB-09 does not satisfy that gate or any later ground, low-drop, subscale, aircraft-integration, flight-test, or controlled-fire gate.

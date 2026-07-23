# ADR-077: Three.js interaction, performance, accessibility, and verification

- **Status**: proposed
- **Date**: 2026-07-23
- **Deciders**:
- **Tags**: operator-console, threejs, accessibility, performance, testing, safety

## Context

Living Three.js workspaces can improve comprehension but can also create inaccessible controls, motion discomfort, misleading interpolation, excessive GPU demand, memory leaks, weak automated coverage, and unsafe coupling between visualization and command execution. Quality must be expressed as measurable gates rather than subjective visual polish.

## Decision

Apply one interaction and verification contract to all Three.js workspace scenes.

### Interaction and command safety

- Pointer picking, keyboard selection, touch, and the adjacent semantic entity list operate on the same selection model.
- Canvas interaction may inspect and stage an action. A semantic HTML control performs authorization, confirmation, and submission.
- Destructive, safety-relevant, or externally consequential actions require clear scope, constraints, expected version, approval state, and a final confirmation outside the canvas.
- Submission returns an idempotent receipt and renders accepted, rejected, acknowledged, executing, outcome-confirmed, outcome-unknown, held, revoked, and failed stages distinctly.
- Optimistic rendering is limited to local view preferences. Operational state changes only after authoritative events.
- Camera motion, selection, filters, and unfinished staged actions are preserved per workspace during tab switching; authorization and confirmation dialogs are not silently preserved.

### Accessibility and motion

- Every canvas has an accessible name, concise current-state description, keyboard instructions, semantic legend, and synchronized HTML representation of essential entities and facts.
- Every selectable or actionable scene object has a stable ID and corresponding focusable HTML control. Keyboard focus and visual selection remain synchronized.
- Focus order, visible focus, announcements, headings, labels, status changes, contrast, target size, zoom, and screen-reader behavior meet the contractually selected current accessibility standard.
- `prefers-reduced-motion` disables ambient animation, replaces travel animations with bounded state transitions, and prevents automatic camera movement. A persistent in-product motion control is provided.
- Color is never the only carrier of state; shape, texture, label, or icon redundantly encode it.
- The static fallback supports all essential inspection and management workflows without WebGL.

### Performance and resilience budgets

Measure on the project’s defined reference field laptop, integrated GPU, and supported browsers. Until benchmark hardware is formally adopted, use these provisional budgets:

| Measure | Budget |
|---|---|
| Initial operator shell JavaScript, gzip | no more than 250 KiB before lazy scene chunks |
| Initial Three.js scene chunk, gzip | no more than 350 KiB |
| Additional scene-specific chunk, gzip | target 150 KiB; hard limit 300 KiB without recorded exception |
| Interactive scene startup after chunk cached | p95 at or below 1.5 s |
| Active animation | p95 at or above 30 FPS; target 60 FPS |
| Main-thread long tasks during steady state | none above 100 ms; fewer than 2/min above 50 ms |
| GPU memory for active scene | target below 256 MiB |
| Tab-switch disposal | GPU/heap returns within 10% of baseline after five cycles |
| Hidden/background scene | zero scheduled frames after the visibility grace period |

Scenes degrade in a declared order: reduce device pixel ratio, shadow quality, particles, model detail, update frequency, and ambient effects; then activate the semantic fallback. Degradation never hides alerts, uncertainty, command state, or essential geometry. WebGL context loss displays a recoverable state, attempts one controlled rebuild, and falls back without losing staged semantic form data.

### Data, time, and visual truthfulness

- Animation samples immutable read-model snapshots and explicit event deltas. It does not fabricate intermediate domain facts.
- Interpolation is visually labeled where it could be mistaken for observation. Extrapolation stops at a bounded horizon and visibly becomes stale or unknown.
- Units, coordinate reference systems, clock quality, observation time, receipt time, lineage, uncertainty, and limitations remain available at the point of use.
- Demo and training scenes use fixed scenario IDs and seeds. They display a persistent simulation watermark and cannot send production commands.
- Telemetry is rate-limited and coalesced before scene updates according to the tiered telemetry policy.

### Verification gates

Each scene requires:

1. Unit tests for state reducers, selectors, geometry derivation, limits, and disposal.
2. Contract tests against `WorkspaceScene`, action gateway, read-model schema, and semantic fallback.
3. Deterministic visual regression at fixed viewport, pixel ratio, clock, seed, camera, and fixture.
4. Browser interaction tests covering tab activation, picking, keyboard equivalence, management workflow, command lifecycle, reduced motion, resize, and fallback.
5. Automated accessibility tests plus manual screen-reader and keyboard review.
6. Performance traces for startup, steady animation, heavy fixture, five repeated tab switches, memory disposal, throttled CPU, and WebGL context loss.
7. Domain review confirming geometry, units, state semantics, uncertainty, constraints, and management actions.
8. Safety and security review for command-capable scenes, including authorization bypass, replay, stale state, misleading outcome, and cross-tenant tests.

CI runs deterministic unit, contract, accessibility, and representative visual tests on every change. The full browser/GPU/performance matrix runs on protected branches and release candidates. Baseline updates require an explained, reviewed change rather than automatic snapshot replacement.

Instrument scene mount time, chunk load, frame time, dropped frames, active object/triangle/draw-call counts, GPU context loss, fallback activation, action-stage latency, and disposal residuals. Do not collect sensitive scene contents or user interaction coordinates unless separately authorized and minimized.

## Consequences

### Positive

- Converts “high quality” and “living” into measurable, repeatable acceptance gates.
- Preserves keyboard, screen-reader, reduced-motion, and non-WebGL operation.
- Detects rendering regressions, resource leaks, stale-state errors, and unsafe action coupling before release.
- Supports graceful operation across a range of field hardware.

### Negative

- Browser/GPU matrices, deterministic rendering, performance labs, and manual accessibility reviews increase delivery cost.
- Strict bundle and frame budgets constrain visual effects and asset fidelity.
- Maintaining equivalent semantic fallbacks requires deliberate duplicate presentation work.

### Neutral

- Performance exceptions may be approved with measured evidence, but safety, state truthfulness, command authorization, and essential accessibility are not waived.
- Visual regression tests supplement rather than replace domain and usability review.

## Links

- [ADR-009](ADR-009-simulation-gated-cyber-physical-delivery.md)
- [ADR-010](ADR-010-zero-trust-identity-and-command-authorization.md)
- [ADR-011](ADR-011-observability-audit-and-evidence-by-design.md)
- [ADR-023](ADR-023-canonical-authenticated-command-envelope.md)
- [ADR-026](ADR-026-tiered-telemetry-and-bounded-backpressure.md)
- [ADR-027](ADR-027-utc-plus-monotonic-time-and-clock-quality.md)
- [ADR-038](ADR-038-secure-software-supply-chain.md)
- [ADR-045](ADR-045-assurance-case-hazard-analysis-and-independent-verification.md)
- [ADR-046](ADR-046-digital-twin-scenario-and-test-evidence.md)
- [ADR-049](ADR-049-privacy-consent-accessibility-and-records.md)
- [ADR-075](ADR-075-threejs-operator-workspace-rendering-platform.md)
- [ADR-076](ADR-076-unique-living-scenes-for-all-operator-workspaces.md)
- [ADR-078](ADR-078-task-oriented-operator-shell-and-design-system.md)
- [ADR-079](ADR-079-resilient-operator-read-model-and-offline-ui-state.md)

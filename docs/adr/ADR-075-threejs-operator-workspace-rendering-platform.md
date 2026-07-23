# ADR-075: Three.js operator workspace rendering platform

- **Status**: proposed
- **Date**: 2026-07-23
- **Deciders**:
- **Tags**: operator-console, threejs, visualization, typescript, webgl

## Context

The operator console exposes fifteen bounded operational workspaces through its left navigation. Static cards and a reused incident-map illustration do not communicate the different state, topology, motion, uncertainty, and control surfaces of those domains. Each selection must feel like a living operational instrument while remaining maintainable, safe, accessible, and usable on ordinary field hardware.

Fifteen unrelated Three.js applications would duplicate render loops, resource management, input handling, visual language, telemetry plumbing, accessibility fallbacks, and tests. A single generic scene with renamed labels would fail the product requirement that every workspace be recognizably unique.

## Decision

Adopt Three.js as the browser rendering layer for bounded, non-authoritative 2.5D and 3D workspace visualizations. Implement one shared rendering platform in `packages/operator-web` and a separately owned scene module for each workspace.

The platform will expose a typed `WorkspaceScene` contract:

```ts
interface WorkspaceScene<TState, TAction> {
  readonly id: WorkspaceId;
  mount(host: HTMLElement, services: SceneServices): void;
  update(state: Readonly<TState>, deltaMs: number): void;
  resize(viewport: Viewport): void;
  setVisibility(visible: boolean): void;
  pick(pointer: NormalizedPointer): SceneSelection | null;
  dispatch(action: TAction): Promise<ActionReceipt>;
  describe(state: Readonly<TState>): AccessibleSceneDescription;
  dispose(): void;
}
```

`SceneServices` provides a renderer, clock, camera helpers, asset registry, theme tokens, deterministic pseudo-random source, reduced-motion state, selection bus, observability hooks, and an action gateway. Scene modules may not create global render loops, attach untracked window listeners, fetch authoritative data directly, or issue commands outside the action gateway.

Use one demand-driven `requestAnimationFrame` scheduler. Only the selected scene is mounted and animated. Background tabs retain serializable view state but consume no animation budget. Continuous animation is permitted only for meaningful live state; otherwise rendering occurs on data, interaction, camera, or resize invalidation. Visibility changes, `document.visibilityState`, and reduced-motion preferences pause or simplify animation.

Use `WebGLRenderer` initially, with capability detection and a static semantic fallback when WebGL is unavailable or context creation fails. WebGPU remains an evaluated future adapter rather than a requirement. The renderer uses capped device pixel ratio, shared geometries/materials, instancing for repeated assets, explicit level of detail, frustum culling, compressed textures where justified, and centralized GPU disposal. Scene transitions must release geometries, materials, textures, controls, observers, and event handlers without leaking GPU or DOM resources.

Three.js visuals are presentation and interaction projections, never systems of record. Every displayed fact carries the state, timestamp, freshness, uncertainty, provenance, and limitation supplied by its owning read model. Animation must not imply a physical outcome that has not been observed. Simulated data is visibly and persistently marked as simulated.

Keep safety-relevant management controls in semantic HTML adjacent to the canvas. Three.js picking may select, inspect, filter, or stage an action, but cannot be the only way to understand state or execute a command. Consequential actions use the existing authorization, confirmation, idempotency, command-stage, and outcome contracts.

The initial module structure is:

```text
src/visualization/
  core/          renderer, scheduler, assets, picking, lifecycle, telemetry
  contracts/     WorkspaceScene, scene state/action types, receipts
  scenes/        one directory per workspace
  overlays/      semantic HTML management panels and legends
  fallbacks/     accessible static/SVG representations
  testing/       deterministic clocks, seeds, fixtures, GPU test adapters
```

Pin Three.js to an exact reviewed version. Load optional controls and examples through explicit imports so production bundles include only adopted modules. Asset licenses, attribution, integrity, provenance, and size budgets are reviewed under the software supply-chain policy.

## Consequences

### Positive

- Provides a coherent, high-quality rendering foundation without making all workspace experiences visually identical.
- Centralizes frame scheduling, GPU lifecycle, observability, accessibility fallback, and failure handling.
- Keeps domain state and command authority outside the rendering engine.
- Allows deterministic scenes and controlled performance degradation for field devices.

### Negative

- Adds a substantial browser dependency, GPU-specific failure modes, and specialized frontend engineering needs.
- Requires careful disposal, profiling, asset governance, and cross-browser testing.
- Semantic HTML overlays and non-WebGL fallbacks increase implementation effort.

### Neutral

- Three.js improves presentation and spatial interaction; it does not make simulated state authoritative or validate operational outcomes.
- Existing SVG and DOM components may remain where they communicate the domain more clearly than 3D.

## Links

- [ADR-011](ADR-011-observability-audit-and-evidence-by-design.md)
- [ADR-014](ADR-014-open-standards-and-dependency-evaluation.md)
- [ADR-016](ADR-016-rust-first-implementation-language.md)
- [ADR-026](ADR-026-tiered-telemetry-and-bounded-backpressure.md)
- [ADR-038](ADR-038-secure-software-supply-chain.md)
- [ADR-046](ADR-046-digital-twin-scenario-and-test-evidence.md)
- [ADR-049](ADR-049-privacy-consent-accessibility-and-records.md)
- [ADR-076](ADR-076-unique-living-scenes-for-all-operator-workspaces.md)
- [ADR-077](ADR-077-threejs-interaction-performance-accessibility-and-verification.md)
- [ADR-078](ADR-078-task-oriented-operator-shell-and-design-system.md)
- [ADR-079](ADR-079-resilient-operator-read-model-and-offline-ui-state.md)

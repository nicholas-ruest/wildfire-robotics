# ADR-078: Task-oriented operator shell and design system

- **Status**: proposed
- **Date**: 2026-07-23
- **Deciders**:
- **Tags**: operator-console, design-system, information-architecture, responsive, accessibility

## Context

The operator console serves fifteen bounded operational workspaces. Its current prototype demonstrates broad domain coverage, but navigation, status cards, actions, tables, maps, alerts, and workspace-specific panels are assembled as large HTML strings with locally defined styles. Similar concepts can therefore acquire different labels or behavior, while different concepts can look deceptively alike.

During an incident, operators must identify what changed, what needs attention, what they are allowed to do, and whether an action produced an authoritative outcome. A visually dense dashboard that gives every value equal weight increases scanning time and error risk. A conventional component library alone will not solve this: the shared vocabulary must encode operational semantics, accessibility, provenance, and command safety.

The console must also work across command-centre displays, field laptops, tablets, keyboard-only use, screen readers, high zoom, reduced motion, and degraded connectivity. Responsive behavior cannot simply hide essential facts or compress fifteen destinations into an unusable icon strip.

## Decision

Adopt a task-oriented operator shell backed by a small, typed design system. The shell and design system are shared presentation infrastructure; bounded contexts continue to own domain language, read models, permissions, and commands.

### Information hierarchy

Every workspace uses the same four-level hierarchy:

1. **Global situation** — incident, operational period, environment, connectivity, data freshness, current UTC time, and signed-in authority.
2. **Workspace attention** — a prioritized queue of safety constraints, alerts, stale or missing data, pending approvals, and failed or indeterminate actions.
3. **Primary task surface** — the map, scene, timeline, graph, table, or form required for the selected operator task.
4. **Selection and action context** — details, provenance, limitations, related evidence, permitted actions, and command lifecycle for the selected entity.

The default view answers “what needs attention now?” before presenting aggregate metrics. Metrics without a decision or task relationship are moved to secondary analysis views.

Navigation is organized into stable, named groups rather than one visually flat list:

- Command: Incident Command, Hazard Intelligence, Predictive Planning, Mission Control
- Operations: Fleet, Vehicle, Station, Logistics, Vegetation, Suppression, Aerial Deployment
- Assurance: Safety, Identity & Access, Robot Care
- Business: Commercial Operations

Routes are addressable and preserve workspace, view, filters, time range, and safe selection state. Deep links must not encode secrets, authorization tokens, confirmations, or unsubmitted command payloads. Browser back and forward navigation behave predictably.

### Shared semantic components

Create framework-agnostic TypeScript contracts and accessible rendered components for:

- `AppShell`, `WorkspaceNavigation`, `SituationHeader`, and `AttentionQueue`
- `DataStateBadge`, `Freshness`, `Provenance`, `Uncertainty`, and `Limitation`
- `Metric`, `EntityTable`, `FilterBar`, `Timeline`, `Legend`, and `DetailsPanel`
- `EmptyState`, `LoadingState`, `ErrorState`, `OfflineState`, and `PermissionState`
- `ActionForm`, `Confirmation`, `ApprovalState`, `CommandReceipt`, and `OutcomeState`
- `Toast` only for transient, non-critical feedback; persistent regions for operational alerts and command results

Components use domain-neutral props and expose slots for bounded-context language. A component may standardize the presentation of “stale,” but the owning context determines the freshness threshold and remediation.

No safety-relevant fact or command is represented only by color, an icon, a tooltip, animation, canvas content, or a transient notification. Tables retain real headings and relationships. Charts and scenes have equivalent names, summaries, units, legends, and inspectable values.

### Tokens and visual language

Define versioned tokens for typography, spacing, density, elevation, focus, touch targets, breakpoints, motion, and semantic state. Tokens replace repeated literal values in workspace styles.

Semantic states are deliberately limited and stable:

| State family | Meaning |
|---|---|
| neutral | known state requiring no judgment |
| current | within the owning context's freshness contract |
| attention | operator review is required |
| warning | bounded risk or degradation exists |
| critical | immediate action or restriction applies |
| unknown | the system cannot establish the value |
| simulated | training, forecast, or synthetic data |

“Green” never means both technically connected and operationally safe. Data quality, health, severity, authorization, and command progress use separate labels and component APIs even if their presentation shares tokens.

Support comfortable and compact density modes without reducing hit targets, focus visibility, or essential labels. The dark theme remains the initial field theme; tokens must permit a tested high-contrast theme without workspace rewrites.

### Responsive and adaptive behavior

Use container-driven layouts with three capability bands rather than device names:

- **wide**: persistent grouped navigation, primary task surface, and context panel;
- **bounded**: collapsible navigation and context panel, with the primary task kept visible;
- **narrow**: task-first single column with explicit navigation and details drawers.

At every band, all fifteen workspaces remain reachable by text label, critical attention remains visible, and consequential actions retain their full scope and confirmation. Horizontal scrolling is allowed inside data tables when a card transformation would destroy row relationships. Layout changes must not reset filters, selection, form entries, or command receipts.

### Delivery and governance

Implement the design system inside `packages/operator-web/src/ui/` with Storybook-equivalent isolated examples generated by the existing build toolchain. Do not adopt an external component framework until a recorded evaluation shows that it meets semantic, bundle, accessibility, theming, and supply-chain requirements.

Migrate the shell first, then shared state and action components, then one workspace at a time. Existing views remain available behind workspace feature flags until parity checks pass.

Changes to a shared semantic state, command component, navigation model, or token require design-system review. Workspace teams may compose shared components and add domain-specific views, but may not fork command lifecycle, freshness, focus, or alert semantics.

## Consequences

### Positive

- Makes attention, task, evidence, and action hierarchy consistent across all workspaces.
- Reduces duplicated markup and CSS while preserving bounded-context language.
- Establishes accessible and responsive behavior as component contracts.
- Prevents visual styling from conflating safety, health, freshness, and authorization.
- Enables incremental migration and visual regression testing.

### Negative

- Requires restructuring the current large string templates and repeated CSS.
- Adds governance and review overhead for shared-component changes.
- A token and component migration may initially slow workspace feature delivery.

### Neutral

- This ADR does not mandate React, Vue, or another UI framework.
- It does not make every workspace look identical; it standardizes operator expectations and safety semantics.
- Three.js scenes remain governed by ADR-075 through ADR-077 and occupy the primary task surface when appropriate.

## Links

- [ADR-001](ADR-001-safety-led-human-command-authority.md)
- [ADR-011](ADR-011-observability-audit-and-evidence-by-design.md)
- [ADR-023](ADR-023-canonical-authenticated-command-envelope.md)
- [ADR-042](ADR-042-configuration-feature-flags-and-runtime-change.md)
- [ADR-049](ADR-049-privacy-consent-accessibility-and-records.md)
- [ADR-075](ADR-075-threejs-operator-workspace-rendering-platform.md)
- [ADR-076](ADR-076-unique-living-scenes-for-all-operator-workspaces.md)
- [ADR-077](ADR-077-threejs-interaction-performance-accessibility-and-verification.md)
- [ADR-079](ADR-079-resilient-operator-read-model-and-offline-ui-state.md)

## Context

ADR 0004 chooses a replaceable hybrid renderer and requires preview/export parity, but its current text is silent about what happens when the preferred deterministic graphics backend is unavailable or cannot execute a valid evaluated instruction. The existing rendering contract already fails readiness with non-retryable `DEPENDENCY_UNAVAILABLE` and forbids publication of incomplete output. The correction must extend that behavior without designing a runtime backend registry prematurely.

## Goals / Non-Goals

**Goals:**

- Define one client-observable distinction between conforming failover and semantic degradation.
- Preserve deterministic preview/export behavior and artifact publication safety.
- Reuse the existing stable dependency error without changing public catalogs.
- Make omission of the fallback decision detectable in the ADR architecture test.

**Non-Goals:**

- Implement backend registration, discovery, selection, readiness, rasterization, or runtime error mapping.
- Choose concrete graphics implementations or expose backend identity to clients.
- Modify project schema, migrations, capabilities, or public requests and responses.
- Rewrite the already archived issue #10 change.

## Decisions

### Conforming local failover is permitted

A future implementation may choose a substitute graphics backend only from locally configured candidates and only after confirming that it supports the complete evaluated scene. Selection follows a deterministic implementation-defined priority shared by frame preview, audiovisual range preview, draft preview, and export. A conforming substitute must preserve the same `EvaluatedScene` semantics and documented visual/audio tolerance; backend identity remains an implementation detail.

Strictly forbidding substitution was rejected because ADR 0004 deliberately makes the graphics boundary replaceable and a conforming local backend does not change client semantics. Network acquisition or ad hoc backend execution was rejected because it violates local-first, path-safety, and deterministic-resource rules.

### Semantic degradation is not fallback

A backend that would omit, approximate, downgrade, reorder, or otherwise reinterpret any evaluated instruction is not conforming. The renderer must not publish a warning-backed or partial success, and it must not feed an incomplete scene to FFmpeg. This applies equally to preview and export.

Permitting explicit degraded output was rejected because it would require a new public warning/capability contract and would undermine the preview/export parity that issue #10 locks.

### Absence of a conforming backend fails closed

If no local conforming backend is ready for the complete scene, readiness or rendering fails with the existing non-retryable `DEPENDENCY_UNAVAILABLE` before graphics rasterization, FFmpeg execution, or artifact publication. No partial or degraded artifact is published. Later runtime milestones must implement this mapping and prove it with injected readiness/capability failures.

A new graphics-specific error was rejected because this correction adds no runtime surface and the canonical catalog already defines the required dependency-unavailable meaning.

### Documentation contract is strengthened

ADR 0004 gains a named fallback subsection. The living hybrid-renderer requirement gains full normative text and two scenarios. The architecture test requires the fallback heading and core phrases, and a focused negative fixture removes only that subsection and must fail with a fallback-specific diagnostic.

## Risks / Trade-offs

- **[Risk]** "Complete scene support" is not yet represented by a capability interface. **Mitigation:** later renderer milestones must define the typed readiness/capability mechanism without weakening this policy.
- **[Risk]** Two conforming backends can differ numerically. **Mitigation:** substitution is allowed only within the same documented output tolerance used for preview/export parity.
- **[Risk]** Reusing `DEPENDENCY_UNAVAILABLE` does not distinguish missing executable from insufficient backend capability. **Mitigation:** keep structured diagnostic details implementation-specific while preserving the stable public code; introduce a new code only through a future governed contract change if clients require that distinction.

## Migration Plan

This documentation-only correction requires no data or contract migration. Sync the modified requirement into the living motion-graphics specification and archive this follow-up change. Rollback is a documentation/test revert; it does not affect project data or artifacts.

## Open Questions

None for this correction. Exact backend candidates, capability representation, and deterministic priority are deferred to the runtime implementation milestone.

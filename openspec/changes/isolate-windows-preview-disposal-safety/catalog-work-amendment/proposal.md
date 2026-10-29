## Catalog verification amendment: proposal

PR233 head2768cd00 fails Windows contracts.test.ts:569 at the unchanged5s guard. Within that one test, DSP and audio-event predecessor projection/expansion each run twice for independent count/hash assertions. Pure Linux/Bun1.4 profiling confirms identical paired outputs, unchanged input, and36 redundant structuredClone calls plus3868 internal serializations; two duplicate expanded JSON representations total17.3MB. This identifies avoidable work, not a proven Windows timing cause or native repair.

Compute each of these two predecessor results once, assert its existing count and pinned hash together, then release the block-local result. Keep the two independent current-catalog expansions, every other predecessor, all negatives, hashes, raw pins and default5s case intact. No cross-test cache or new helper abstraction.

Only apps/agent-bridge/tests/contracts.test.ts and this existing active change's planning/evidence may change. Modified capability: contract-governance. No production, native/disposal/lifecycle, fixtures/catalogs, dependencies, workflows, profile, timeout or skip changes. No merge/rebase of main, PR229 changes, denied publications, approval, merge or deployment.

Keep isolate-windows-preview-disposal-safety ACTIVE: attribution/correction conditions remain unmet. This amendment must not archive it or claim overall diagnostic completion. Publication remains an investigation draft with the known OpenSpec rejection disclosed.

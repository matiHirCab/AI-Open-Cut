## Decision

Immediately realpath each newly created owned root before child paths, manifest keys or configuration. Change only tests/artwork-durable-backends.test.ts setup and tests/artwork-durable-workflow.ts root creation; parameterize the private setup parent for regression coverage. No new abstraction.

## Boundaries / alternatives

Keep artwork-durable-probe.mjs and fake-media-tool.mjs byte-identical. Do not canonicalize arbitrary probe inputs into acceptance or relax containment, identity, hash, type, size or format controls. Root-only manifest normalization still fails the controlled reproduction; relaxing the probe masks escapes and is rejected.

Create an owned physical parent plus ancestor alias (Windows junction, otherwise directory symlink). Canonical producer PNG/WAV paths must pass with empty PATH; allowlisted explicit alias paths, foreign files and same-root/foreign linked child directories must fail. Existing leaf-link and malformed/changed/unknown/oversize/configuration controls remain, with no new skips.

## Verification / risk

Red-before-green regression; full bridge checks; separate source/package hermetic and genuine proof, plus aliased TMPDIR/no-ambient controls. Linux does not prove Windows/macOS. No production/public/persisted/security boundary changes, dependency or migration. Fail explicitly; no fallback. Rollback only the private test diff; preserve raw failed CI/history.

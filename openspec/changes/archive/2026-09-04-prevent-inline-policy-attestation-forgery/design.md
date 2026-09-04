# Context

The current completion marker is written by the same workflow `run` body whose integrity it is intended to prove. Bash control flow can therefore neutralize a detected workflow mutation and continue to the marker write. The repository already treats the pinned Moon task and structural validator as reviewed code, so the marker should be produced there only after the entire validation sequence succeeds.

# Goals / Non-Goals

**Goals:**

- Prevent `|| true` or another wrapper around the canonical workflow command from producing the policy marker.
- Define and validate the complete Moon policy task sequence.
- Emit no output after any normalization, test, OpenSpec, workflow, or Moon-task validation failure.
- Preserve local execution when GitHub's output file is unavailable.

**Non-Goals:**

- Make a PR-controlled workflow cryptographically tamper-proof without a trusted external required workflow or repository rule.
- Change public APIs, contracts, application behavior, fixtures, goldens, or persisted data.

# Decisions

## Make the workflow command single-purpose

Keep the policy step identifier `policy` and restore its exact command to `moon run openspec-validate`. The workflow may consume the resulting step output but may not write it directly.

## Attest from the final Moon command

Define the exact `openspec-validate` task order as normalization check, policy tests, strict pinned OpenSpec validation, and finally `bun run scripts/validate-ci-gates.ts --attest-github-output`. Join every command with explicit `&&` so the final process is unreachable after any earlier nonzero exit. The final process validates both the workflow and this Moon task before appending `validated=true` to `GITHUB_OUTPUT`.

When `--attest-github-output` is present outside GitHub Actions and no output path exists, validation still succeeds without writing a marker. An available but unwritable output path remains a hard failure.

## Validate before invoking the output writer

Add a pure Moon-config validator and a validation-plus-attestation function whose output callback runs only after both structural inputs pass. The CLI loads `.github/workflows/bun-ci.yml` and `moon.yml`, validates both, then appends the exact marker only when requested.

This makes the masked workflow mutation fail inside Moon before the final attesting invocation, so `|| true` outside Moon cannot manufacture the missing marker.

# Risks / Trade-offs

- **Moon task order becomes rigid.** Any legitimate policy-stage change requires synchronized specification, validator, workflow, and tests.
- **The trust boundary remains local.** Replacing the entire verifier with code that directly forges outputs remains an explicit branch-review and external-protection concern.

# Migration Plan

Land the workflow, Moon task, validator, tests, documentation, and living requirement together. Verify every repository gate, synchronize the repository-validation delta, and archive this follow-up. Maintainers continue requiring `Motion-graphics foundation parity`.

# Open Questions

None.

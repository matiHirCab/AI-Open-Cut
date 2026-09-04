# Contract and render parity CI gates

OpenCut publishes three stable motion-graphics foundation statuses:

- `Contract parity` (`contract-parity`) runs the canonical Rust, Serde, TypeScript, Zod, and MCP fixture-governance boundary.
- `Render parity` (`render-parity`) runs deterministic Linux preview, audiovisual range, export, and lifecycle conformance, validates the report-only benchmark observation, and uploads that exact report.
- `Motion-graphics foundation parity` (`foundation-parity`) always runs after OpenSpec policy validation and both leaf gates reach a terminal result, logs all three results plus the policy completion attestation, and fails unless all three are exactly `success` and the attestation is exactly `true`. Maintainers should configure this aggregate status as the required branch-protection check after the workflow is present on the default branch.

The render gate uses FFmpeg, FFprobe, and DejaVu Sans from the Ubuntu runner packages. Its golden harness is fail-closed through `OPENCUT_GOLDEN_REQUIRED=1`; timing and memory observations remain report-only rather than universal performance budgets. Required CI is verification-only: it must never set `OPENCUT_UPDATE_GOLDENS` or `OPENCUT_CAPTURE_GOLDENS_TO` at workflow, job, or critical-step scope. Updating the canonical goldens or capturing an alternate comparison set remains an explicitly chosen local maintenance operation.

The OpenSpec policy job and both leaf gates use closed reviewed step sequences. OpenSpec validation installs Bun 1.4.0 and Moon 2.3.3 from explicit action inputs before repository checkout, then performs full-history checkout and invokes `bun --config=/dev/null --no-env-file run scripts/run-ci-policy.ts --attest-github-output`. The empty bootstrap config and disabled dotenv loading prevent checkout-controlled `bunfig.toml` preloads or `.env` process controls from running before preflight. The bootstrap validates the workflow and complete Bun/Moon/proto boundary before starting Moon, launches exactly `moon run root:openspec-validate` without a shell, and removes `GITHUB_OUTPUT` from the child environment. The reviewed Moon task runs normalization, policy unit and real-Moon integration tests, strict OpenSpec validation, and a final validation-only structural check in exact order, joined by explicit `&&`; every Bun invocation selects the already-validated `bunfig.toml` and disables dotenv loading. Only the bootstrap may emit `validated=true`, and only after the Moon child exits successfully. Its sole job output remains bound to that step output. Contract parity contains only checkout, pinned toolchain setup, dependency installation, and the canonical contract command. Render parity contains only checkout, deterministic dependency installation, pinned toolchain setup, native conformance, strict report validation, and report upload. Additional, duplicate, replaced, or reordered steps are rejected because earlier steps can modify repository evidence or persist environment variables through `GITHUB_ENV`.

The OpenSpec job result alone is insufficient evidence because GitHub Actions can report a successful conclusion after `continue-on-error` or shell control flow masks a failure. The bootstrap owns the private step-output path and does not expose it to Moon. A failed preflight, process launch, skipped or neutralized task, or nonzero Moon exit therefore leaves the output absent, and the aggregate rejects that missing attestation even if `needs.openspec.result` is `success`.

The preflight also enforces archive-only OpenSpec merge readiness. `openspec/changes` and its canonical `archive` child must be ordinary directories, and every other direct entry—including a file, directory, or symbolic link—is treated as an unarchived change. Such entries are valid during local authoring, but the protected gate rejects all of them before starting Moon and emits no policy attestation until every completed change has been synchronized, verified, and archived.

The canonical `bunfig.toml`, root Moon project, and proto pins are part of the protected execution boundary. The validator requires the reviewed Bun dependency-age policy exactly, while the project disables task inheritance and may not declare project-wide environment, platform, Docker, or toolchain overrides. It also fixes the root workspace mapping, Moon toolchains, and exact `.prototools` versions, rejects remote or alternate configuration, and inventories `.moon/tasks`. Global task files may define tasks for non-root projects, but may not inject a global environment, inherit implicit execution options, or extend unreviewed configuration. Because the bootstrap excludes Bun configuration and dotenv loading before performing these checks, and the validated task keeps dotenv loading disabled, a `bunfig.toml` preload, local `.env` `BASH_ENV`, forged `PATH`, and equivalent project configuration cannot execute first.

This remains a reviewed-code guard rather than a cryptographic trust boundary. The repository is personal and does not use an organization-owned required workflow. Deliberately replacing the workflow and bootstrap together remains subject to code review; `main` must require `@matiHirCab` CODEOWNER approval for the listed policy files in addition to requiring the aggregate status. `pull_request_target` is not used because this policy executes pull-request code.

The workflow, OpenSpec policy job, and all three parity jobs must omit job-inherited `env` maps entirely, including empty maps. This isolates the reviewed commands from startup hooks and execution overrides such as `BASH_ENV`, `PATH`, `LD_PRELOAD`, `LD_AUDIT`, and `NODE_OPTIONS`, as well as from apparently innocuous inherited metadata. Jobs outside the protected boundary may still declare their own environment. Any variable genuinely required by a protected job must be added to the exact environment map of its authorized step through a coordinated update to the applicable OpenSpec requirement, policy validator, and tests.

## Local reproduction

Run the contract boundary from `apps/agent-bridge`:

```sh
bun install --frozen-lockfile
bun run contracts:check
```

Run workflow-policy and OpenSpec validation from the repository root:

```sh
bun --config=bunfig.toml --no-env-file test scripts/validate-ci-gates.test.ts scripts/run-ci-policy.test.ts scripts/run-ci-policy.integration.test.ts
bun --config=bunfig.toml --no-env-file run scripts/validate-ci-gates.ts
moon run root:openspec-validate
bun --config=NUL --no-env-file run scripts/run-ci-policy.ts # Windows
# bun --config=/dev/null --no-env-file run scripts/run-ci-policy.ts # Linux
```

On Linux, configure explicit deterministic render dependencies and an absolute report destination before running native parity:

```sh
export OPENCUT_FFMPEG_PATH=ffmpeg
export OPENCUT_FFPROBE_PATH=ffprobe
export OPENCUT_GOLDEN_REPORT_PATH="$(pwd)/target/render-baseline-linux.json"
export OPENCUT_GOLDEN_REQUIRED=1
export OPENCUT_TEST_FONT_PATH=/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf
mkdir -p "$(dirname "$OPENCUT_GOLDEN_REPORT_PATH")"
cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact
cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact
cargo test -p opencut-editor-core renderer::golden::validate_external_performance_report -- --ignored --exact
```

The policy validator parses the workflow, canonical `bunfig.toml`, Moon project/workspace/toolchains, `.prototools`, global-task inventory, and direct OpenSpec change inventory structurally. It fails if the archive-only state, bootstrap isolation flags, Bun configuration, bootstrap versions or order, a stable job, aggregate dependency, result or attestation assertion, closed protected sequence, exact Moon policy task, mandatory real-Moon regression, approved step property or environment, authoritative command, working directory, failure-propagation rule, deterministic render setting, validation order, or exact upload path is weakened. Critical steps retain exact fail-closed bodies and cannot opt into `continue-on-error`. The native conformance step declares exactly its five approved environment variables, the report validator only its report path, and the aggregate exactly its three result bindings plus the completion output. A failed, cancelled, skipped, or unattested policy execution therefore reaches the same aggregate as a parity failure.

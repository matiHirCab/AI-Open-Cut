## Context

The reviewed bootstrap is invoked after checkout with `bun run`. Bun 1.4.0 reads repository `bunfig.toml` and dotenv files before the TypeScript entry point runs, so a pull request can execute a preload, write the expected GitHub output, and terminate successfully before structural validation. The protected Moon task also invokes Bun repeatedly, while its real-Moon regression is currently listed only as an input and therefore is not run by CI.

## Goals / Non-Goals

**Goals:**

- Prevent checkout-controlled Bun configuration and dotenv files from executing before bootstrap validation.
- Validate the canonical Bun configuration before Moon starts and make every protected Bun invocation explicit.
- Execute the real-Moon startup-hook regression in the protected Ubuntu task.
- Preserve the existing policy output and aggregate branch-protection interface.

**Non-Goals:**

- Introduce an organization workflow, privileged pull-request event, or cryptographic attestation.
- Prohibit developers from keeping an ignored local `.env` for application development.
- Change application behavior, contracts, renderer output, fixtures, goldens, or persisted data.

## Decisions

### Start the bootstrap with no checkout-controlled Bun inputs

The workflow will invoke Bun with `--config=/dev/null --no-env-file`. `/dev/null` supplies an empty trusted config on the required Ubuntu runner and `--no-env-file` prevents automatic dotenv loading. Merely validating or CODEOWNING `bunfig.toml` was rejected because Bun would interpret it before the validator could run.

### Validate and explicitly reuse the canonical Bun config after preflight

The bootstrap source inventory will require the checked-in `bunfig.toml` to match its reviewed content exactly. Each Bun command in the Moon task will use `--config=bunfig.toml --no-env-file`, so the already-validated config is explicit and ignored local dotenv files cannot change protected execution. Removing `bunfig.toml` entirely was rejected because its dependency-age policy remains useful elsewhere in the repository.

### Execute the real-Moon regression inside the protected task

The integration test will join the existing `bun test` command. It creates an independent temporary Moon workspace and therefore does not recursively invoke the repository's protected task. Listing it only as an input was rejected because inputs affect invalidation, not test execution.

## Risks / Trade-offs

- **Bun flag behavior differs by platform.** → CI uses `/dev/null` on Ubuntu; local Windows reproduction uses `NUL`, while the cross-platform Moon task uses the reviewed repository config path.
- **The Linux integration test adds process startup cost.** → Keep it as one isolated test and run it in the already-required policy job.
- **Global runner configuration remains trusted.** → Explicit CLI config and dotenv flags prevent repository files from selecting those inputs; setup actions and the runner image remain part of the documented trust base.

## Migration Plan

Land the workflow command, Bun-source validation, Moon task, CODEOWNERS, tests, docs, and specification together. Verify on Ubuntu 24.04, synchronize the living requirement, and archive the follow-up. Rollback restores the prior workflow command and Moon task as one unit; no data migration is required.

## Open Questions

None.

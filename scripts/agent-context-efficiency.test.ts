import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { TOML } from "bun";

const read = (path: string) =>
  readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const pluginIds = [
  "canva@openai-curated",
  "figma@openai-curated",
  "vercel@openai-curated",
  "sites@openai-bundled",
  "documents@openai-primary-runtime",
  "spreadsheets@openai-primary-runtime",
  "presentations@openai-primary-runtime",
  "pdf@openai-primary-runtime",
  "template-creator@openai-primary-runtime",
];

function assertConfig(source: string) {
  expect(TOML.parse(source)).toEqual({
    model: "gpt-6-astra",
    model_reasoning_effort: "medium",
    plan_mode_reasoning_effort: "medium",
    plugins: Object.fromEntries(
      pluginIds.map((id) => [id, { enabled: false }])
    ),
  });
}

function section(source: string, heading: string) {
  const start = source.indexOf(`## ${heading}\n`);
  expect(start).toBeGreaterThanOrEqual(0);
  const next = source.indexOf("\n## ", start + 1);
  return source.slice(start, next === -1 ? undefined : next);
}

const safeguards: Record<string, string[]> = {
  "Compatibility and migrations": [
    "stable error codes and retryability aligned with the canonical catalog",
    "Follow ADR 0002 and `contracts/contract-ownership-v1.json`",
    "run `bun run contracts:check`",
    "obtain review from the designated CODEOWNER",
    "requires a new major contract plus an explicit migration path",
    "deterministic migration tests for current state and retained undo/redo history",
    "Perform migrations under the project lock",
    "Reject unknown future schema versions",
  ],
  "Mandatory spec-driven development": [
    "All implementation work MUST follow the OpenSpec lifecycle",
    "Bug fixes and refactors are not exempt.",
    "then obtain explicit user or reviewer approval before implementation",
    "Implement only through the approved change's `tasks.md`",
    "Add or update tests for every changed scenario",
    "Each normative requirement MUST have automated coverage",
    "Keep code, contracts, fixtures, migrations, documentation, design decisions, delta specs, and tasks synchronized",
    "plus all affected formatting, linting, type, unit, integration, migration, and smoke checks",
    "A failed or skipped required check blocks completion",
    "Use `$openspec-verify-change` after implementation",
    "Archive the verified change with `$openspec-archive-change`",
  ],
  "Ownership boundaries": [
    "ADR 0003",
    "Any new dependency edge requires the ADR and the editor-core architecture test to change",
    "must not add parallel validation",
    "`crates/editor-core` owns domain models",
    "`apps/headless` is the typed JSON-lines process transport",
    "`apps/agent-bridge` owns application workflows",
    "`apps/kokoro-tts` is a replaceable, CPU-only speech provider worker",
    "`apps/desktop` owns presentation and interaction",
    "`contracts` contains canonical cross-language catalogs and fixtures",
    "`editor-core` must never depend on those outer layers",
  ],
  "Testing and maintenance": [
    "Rust formatting, workspace-wide strict Clippy, workspace tests, TypeScript typecheck/lint/unit tests, MCP integration, packaged smoke, and the hermetic Python worker tests",
    "Do not add crate-wide or workspace-wide warning suppression",
    "Preserve unrelated working-tree changes",
    "do not commit secrets, local data, dependency caches, or generated build output",
  ],
};

function assertSafeguards(source: string) {
  for (const [heading, phrases] of Object.entries(safeguards)) {
    const content = section(source.replaceAll("\r\n", "\n"), heading);
    for (const phrase of phrases) {
      expect(content).toContain(phrase);
    }
  }
}

const unsupportedRemovalClaim =
  /(?:disables (?:all )?nine unrelated design\/document plugins|all nine plugins are disabled in desktop tasks)/i;

function assertPluginScope(source: string) {
  for (const phrase of [
    "local-marketplace overrides",
    "Remote/workspace-managed plugins may remain available",
    "accepted limitation",
    "CLI-only evidence does not prove desktop availability",
  ]) {
    expect(source.includes(phrase)).toBe(true);
  }
  expect(unsupportedRemovalClaim.test(source)).toBe(false);
}

const verificationSteps = [
  "Run required implementation checks",
  "Run the protected gate before archival",
  "Verify implementation conformance",
  "Synchronize and archive",
  "Run the protected gate after archival",
];

function assertVerificationOrder(source: string) {
  const guidance = section(
    source.replaceAll("\r\n", "\n"),
    "Verification order"
  );
  let previous = -1;
  for (const step of verificationSteps) {
    const position = guidance.indexOf(step);
    expect(position).toBeGreaterThan(previous);
    previous = position;
  }
  for (const phrase of [
    "only this active change",
    "not a passed gate",
    "Any other failure blocks archival",
    "must pass before declaring completion",
    "Do not weaken checks or fabricate an attestation",
  ]) {
    expect(guidance.includes(phrase)).toBe(true);
  }
}

describe("project agent context policy", () => {
  test("sets only approved local model and plugin overrides", () => {
    assertConfig(read(".codex/config.toml"));
  });

  test.each([
    ["medium", "xhigh"],
    ["canva@openai-curated", "canva@wrong-marketplace"],
    ["enabled = false", "enabled = true"],
  ])("rejects configuration drift from %s to %s", (before, after) => {
    expect(() =>
      assertConfig(read(".codex/config.toml").replace(before, after))
    ).toThrow();
  });

  test("distinguishes local overrides from remote desktop availability", () => {
    assertPluginScope(read("docs/spec-driven-development.md"));
  });

  test.each([
    "This disables nine unrelated design/document plugins.",
    "All nine plugins are disabled in desktop tasks.",
  ])("rejects unsupported complete-removal claim: %s", (claim) => {
    expect(() =>
      assertPluginScope(`${read("docs/spec-driven-development.md")}\n${claim}`)
    ).toThrow();
  });

  test("requires implementation verification before archival and final gate afterward", () => {
    assertVerificationOrder(read("docs/spec-driven-development.md"));
  });

  test("rejects moving the final gate before archival", () => {
    const guide = read("docs/spec-driven-development.md")
      .replace("Run the protected gate before archival", "TEMP_STEP")
      .replace(
        "Run the protected gate after archival",
        "Run the protected gate before archival"
      )
      .replace("TEMP_STEP", "Run the protected gate after archival");
    expect(() => assertVerificationOrder(guide)).toThrow();
  });

  test.each([
    "Any other failure blocks archival",
    "must pass before declaring completion",
  ])("rejects removed final-check safeguard: %s", (phrase) => {
    expect(() =>
      assertVerificationOrder(
        read("docs/spec-driven-development.md").replace(phrase, "")
      )
    ).toThrow();
  });

  test("retains mandatory safeguards in the owning instruction sections", () => {
    assertSafeguards(read("AGENTS.md"));
  });

  test.each([
    "then obtain explicit user or reviewer approval before implementation",
    "A failed or skipped required check blocks completion",
    "Use `$openspec-verify-change` after implementation",
  ])("rejects removal of safeguard: %s", (phrase) => {
    expect(() =>
      assertSafeguards(read("AGENTS.md").replace(phrase, ""))
    ).toThrow();
  });

  test("documents focused reads, full failure evidence, and invalidation", () => {
    const guidance = section(
      read("AGENTS.md").replaceAll("\r\n", "\n"),
      "Efficient context and verification"
    );
    for (const phrase of [
      "filenames and requirement headings",
      "Exclude `openspec/changes/archive/` from routine searches",
      "history is needed",
      "validators must still inspect their full required scope",
      "Reuse already-read instructions",
      "changed or are no longer in context",
      "full output in uncommitted local logs",
      "command, exit status, summary, relevant failures, and log location",
      "inputs, toolchain, and environment remain unchanged",
      "new evidence invalidates it or an explicit rerun is required",
      "All required final checks remain mandatory",
    ]) {
      expect(guidance).toContain(phrase);
    }
  });

  test("documents reversible plugins and honest manual verification", () => {
    const guide = read("docs/spec-driven-development.md");
    for (const phrase of [
      ".codex/config.toml",
      "enabled = true",
      "fresh OpenCut task",
      "another project",
      "managed override",
      "global configuration",
      "static tests do not prove",
      "do not claim measured savings",
      "bun test scripts/agent-context-efficiency.test.ts",
      "only its ordinary `archive/` directory",
    ]) {
      expect(guide).toContain(phrase);
    }
  });
});

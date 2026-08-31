import { readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

const OPEN_SPEC_VERSION = "1.5.0";
const PINNED_COMMAND = `bunx @fission-ai/openspec@${OPEN_SPEC_VERSION}`;
const SKILLS_DIRECTORY = join(".codex", "skills");
const CHECK_ONLY = process.argv.includes("--check");

const aliases = new Map([
  ["/opsx:apply", "$openspec-apply-change"],
  ["/opsx:archive", "$openspec-archive-change"],
  ["/opsx:continue", "$openspec-continue-change"],
  ["/opsx:explore", "$openspec-explore"],
  ["/opsx:ff", "$openspec-ff-change"],
  ["/opsx:new", "$openspec-new-change"],
  ["/opsx:onboard", "$openspec-onboard"],
  ["/opsx:propose", "$openspec-propose"],
  ["/opsx:verify", "$openspec-verify-change"],
]);

const unpinnedCommand =
  /\bopenspec (?=--version|archive|context|doctor|instructions|list|new|schemas|show|status|store|update|validate)/g;
const versionedCommand = /bunx @fission-ai\/openspec@[^\s`]+/g;

function normalize(content: string): string {
  let normalized = content;
  for (const [alias, skill] of aliases) {
    normalized = normalized.replaceAll(alias, () => skill);
  }
  normalized = normalized
    .replaceAll(
      "compatibility: Requires openspec CLI.",
      `compatibility: Requires Bun to run @fission-ai/openspec@${OPEN_SPEC_VERSION}.`
    )
    .replaceAll(
      "Use the **AskUserQuestion tool** (open-ended, no preset options) to ask:",
      "Ask the user an open-ended question:"
    )
    .replaceAll(
      "use the **AskUserQuestion tool** to let the user select",
      "ask the user to select"
    )
    .replaceAll(
      "Use the **AskUserQuestion tool** to let the user select",
      "Ask the user to select"
    )
    .replaceAll(
      "use **AskUserQuestion tool** to clarify",
      "ask the user to clarify"
    )
    .replaceAll(
      "Use **AskUserQuestion tool** to clarify",
      "Ask the user to clarify"
    )
    .replaceAll(
      "Use **AskUserQuestion tool** to confirm user wants to proceed",
      "Ask the user to confirm they want to proceed"
    )
    .replaceAll(
      "Use **AskUserQuestion tool** with multi-select to let user choose changes:",
      "Ask the user to choose one or more changes:"
    )
    .replaceAll(
      "Use **AskUserQuestion tool** with a single confirmation:",
      "Ask the user for a single confirmation:"
    )
    .replaceAll(
      "Use the **TodoWrite tool** to track progress through the artifacts.",
      "Track progress through the artifacts in the task plan."
    )
    .replaceAll(
      'If user chooses sync, use Task tool (subagent_type: "general-purpose", prompt: "Use Skill tool to invoke openspec-sync-specs for change \'<name>\'. Delta spec analysis: <include the analyzed delta spec summary>"). Proceed to archive regardless of choice.',
      "If the user chooses sync, follow `$openspec-sync-specs` for change `<name>` using the delta spec summary above. Proceed to archive after the sync workflow completes."
    )
    .replaceAll(
      "# if (Get-Command openspec -ErrorAction SilentlyContinue) { openspec --version } else { echo \"CLI_NOT_INSTALLED\" }",
      `# ${PINNED_COMMAND} --version`
    )
    .replaceAll(
      `# if (Get-Command openspec -ErrorAction SilentlyContinue) { ${PINNED_COMMAND} --version } else { echo "CLI_NOT_INSTALLED" }`,
      `# ${PINNED_COMMAND} --version`
    )
    .replaceAll(
      "OpenSpec CLI is not installed. Install it first, then come back to `$openspec-onboard`.",
      "The pinned OpenSpec CLI could not run. Install Bun or resolve the reported package error, then come back to `$openspec-onboard`."
    );

  return normalized
    .replace(versionedCommand, PINNED_COMMAND)
    .replace(unpinnedCommand, `${PINNED_COMMAND} `);
}

const skillDirectories = (await readdir(SKILLS_DIRECTORY, { withFileTypes: true }))
  .filter((entry) => entry.isDirectory() && entry.name.startsWith("openspec-"))
  .map((entry) => entry.name)
  .sort();

const failures: string[] = [];
for (const directory of skillDirectories) {
  const path = join(SKILLS_DIRECTORY, directory, "SKILL.md");
  const content = await readFile(path, "utf8");
  const normalized = normalize(content);

  if (!content.includes(`generatedBy: "${OPEN_SPEC_VERSION}"`)) {
    failures.push(`${path}: generatedBy must be ${OPEN_SPEC_VERSION}`);
  }

  if (CHECK_ONLY) {
    if (normalized !== content) {
      failures.push(`${path}: run bun run scripts/normalize-openspec-workflows.ts`);
    }
  } else if (normalized !== content) {
    await writeFile(path, normalized, "utf8");
  }
}

const moonConfig = await readFile("moon.yml", "utf8");
if (!moonConfig.includes(`${PINNED_COMMAND} validate --all --strict --no-interactive`)) {
  failures.push(`moon.yml: OpenSpec validation must use ${PINNED_COMMAND}`);
}

const contributorGuide = await readFile(
  join("docs", "spec-driven-development.md"),
  "utf8"
);
if (
  !contributorGuide.includes(`OpenSpec ${OPEN_SPEC_VERSION}`) ||
  !contributorGuide.includes(
    `${PINNED_COMMAND} validate --all --strict --no-interactive`
  )
) {
  failures.push(
    `docs/spec-driven-development.md: documented commands must use ${OPEN_SPEC_VERSION}`
  );
}

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(failure);
  }
  process.exit(1);
}

console.log(
  CHECK_ONLY
    ? `OpenSpec workflows are normalized and pinned to ${OPEN_SPEC_VERSION}`
    : `Normalized OpenSpec workflows for ${OPEN_SPEC_VERSION}`
);

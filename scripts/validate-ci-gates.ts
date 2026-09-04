import { existsSync, lstatSync, readdirSync, readFileSync } from "node:fs";
import { extname, join } from "node:path";

const DEFAULT_WORKFLOW = ".github/workflows/bun-ci.yml";
const DEFAULT_BUN_CONFIG = "bunfig.toml";
const DEFAULT_MOON_CONFIG = "moon.yml";
const DEFAULT_MOON_WORKSPACE_CONFIG = ".moon/workspace.yml";
const DEFAULT_MOON_TOOLCHAINS_CONFIG = ".moon/toolchains.yml";
const DEFAULT_MOON_TASKS_DIRECTORY = ".moon/tasks";
const DEFAULT_PROTO_CONFIG = ".prototools";
const DEFAULT_OPENSPEC_CHANGES_DIRECTORY = "openspec/changes";
const OPENSPEC_ARCHIVE_DIRECTORY = "archive";
const AGENT_BRIDGE_DIRECTORY = "apps/agent-bridge";
const REPORT_PATH = "target/render-baseline-linux.json";
const ABSOLUTE_REPORT_PATH = "${{ github.workspace }}/target/render-baseline-linux.json";
const FOUNDATION_CONDITION = "${{ always() }}";
const OPENSPEC_COMMAND =
  "bun --config=/dev/null --no-env-file run scripts/run-ci-policy.ts --attest-github-output";
const OPENSPEC_TASK_COMMAND = `bun --config=bunfig.toml --no-env-file run scripts/normalize-openspec-workflows.ts --check &&
bun --config=bunfig.toml --no-env-file test scripts/validate-ci-gates.test.ts scripts/run-ci-policy.test.ts scripts/run-ci-policy.integration.test.ts &&
bun --config=bunfig.toml --no-env-file x @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive &&
bun --config=bunfig.toml --no-env-file run scripts/validate-ci-gates.ts`;
const OPENSPEC_TASK_INPUTS = [
  ".codex/skills/openspec-*/SKILL.md",
  "docs/spec-driven-development.md",
  "openspec/**/*",
  "scripts/normalize-openspec-workflows.ts",
  "scripts/validate-ci-gates.ts",
  "scripts/validate-ci-gates.test.ts",
  "scripts/run-ci-policy.ts",
  "scripts/run-ci-policy.test.ts",
  "scripts/run-ci-policy.integration.test.ts",
  ".github/workflows/bun-ci.yml",
  "bunfig.toml",
  ".prototools",
  "moon.*",
  ".moon/*",
  ".moon/tasks/**/*",
] as const;
const MOON_CONFIG_EXTENSIONS = [".hcl", ".json", ".jsonc", ".pkl", ".toml", ".yaml", ".yml"];
const PROTO_CONFIG = `# proto pins tool versions workspace-wide.
# Every developer and CI machine gets the exact same versions automatically.
moon = "2.3.3"
bun  = "1.4.0"
rust = "1.97.0"`;
const BUN_CONFIG = `[install]
minimumReleaseAge = 604800 # 7d`;
const FORBIDDEN_GOLDEN_MODES = [
  "OPENCUT_UPDATE_GOLDENS",
  "OPENCUT_CAPTURE_GOLDENS_TO",
] as const;

const NATIVE_PARITY_COMMAND = `mkdir -p "$(dirname "$OPENCUT_GOLDEN_REPORT_PATH")"
cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact
cargo test -p opencut-headless native_render_lifecycle_survives_edit_undo_redo_reopen_and_isolates_drafts -- --exact`;

const FOUNDATION_COMMAND = `echo "OpenSpec validation: $OPENSPEC_RESULT"
echo "OpenSpec policy attested: $OPENSPEC_POLICY_VALIDATED"
echo "Contract parity: $CONTRACT_PARITY_RESULT"
echo "Render parity: $RENDER_PARITY_RESULT"
if [ "$OPENSPEC_RESULT" != "success" ] || [ "$OPENSPEC_POLICY_VALIDATED" != "true" ] || [ "$CONTRACT_PARITY_RESULT" != "success" ] || [ "$RENDER_PARITY_RESULT" != "success" ]; then
  echo "Foundation parity requires attested policy validation and both leaf gates to succeed." >&2
  exit 1
fi`;

type UnknownRecord = Record<string, unknown>;

export interface MoonConfigurationSource {
  path: string;
  source: string;
}

export interface MoonPolicySources {
  bun: string;
  project: string;
  workspace: string;
  toolchains: string;
  proto: string;
  globalTasks: readonly MoonConfigurationSource[];
  unexpectedConfigurations: readonly string[];
  activeChanges: readonly string[];
}

function record(value: unknown, label: string): UnknownRecord {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value as UnknownRecord;
}

function steps(job: UnknownRecord, label: string): UnknownRecord[] {
  const value = job.steps;
  if (!Array.isArray(value)) {
    throw new Error(`${label}.steps must be an array`);
  }
  return value.map((step, index) => record(step, `${label}.steps[${index}]`));
}

function requiredJob(jobs: UnknownRecord, id: string): UnknownRecord {
  if (!(id in jobs)) {
    throw new Error(`workflow must define jobs.${id}`);
  }
  return record(jobs[id], `jobs.${id}`);
}

function requiredStep(
  jobSteps: UnknownRecord[],
  name: string,
  label: string
): { step: UnknownRecord; index: number } {
  const index = jobSteps.findIndex((step) => step.name === name);
  if (index === -1) {
    throw new Error(`${label} must define step ${JSON.stringify(name)}`);
  }
  return { step: jobSteps[index]!, index };
}

function requireExactKeys(
  value: UnknownRecord,
  expectedKeys: string[],
  label: string
): void {
  const actualKeys = Object.keys(value).sort();
  const sortedExpectedKeys = [...expectedKeys].sort();
  if (
    actualKeys.length !== sortedExpectedKeys.length ||
    actualKeys.some((key, index) => key !== sortedExpectedKeys[index])
  ) {
    throw new Error(`${label} must contain exactly the approved properties`);
  }
}

function requireAllowedKeys(
  value: UnknownRecord,
  allowedKeys: readonly string[],
  label: string
): void {
  const unexpected = Object.keys(value).filter((key) => !allowedKeys.includes(key));
  if (unexpected.length > 0) {
    throw new Error(`${label} contains unapproved properties: ${unexpected.sort().join(", ")}`);
  }
}

function requireExactStepSequence(
  jobSteps: UnknownRecord[],
  expectedNames: Array<string | undefined>,
  label: string
): void {
  if (jobSteps.length !== expectedNames.length) {
    throw new Error(`${label} must contain exactly its approved step sequence`);
  }
  expectedNames.forEach((expectedName, index) => {
    if (jobSteps[index]!.name !== expectedName) {
      throw new Error(`${label}.steps[${index}] must be ${JSON.stringify(expectedName)}`);
    }
  });
}

function requireExactStringArray(
  value: unknown,
  expected: readonly string[],
  label: string
): void {
  if (
    !Array.isArray(value) ||
    value.length !== expected.length ||
    value.some((item, index) => item !== expected[index])
  ) {
    throw new Error(`${label} must contain exactly the approved ordered values`);
  }
}

function normalizedCommand(value: unknown): string {
  return typeof value === "string" ? value.replaceAll("\r\n", "\n").trim() : "";
}

function requireExactCommand(step: UnknownRecord, expected: string, label: string): void {
  if (normalizedCommand(step.run) !== expected) {
    throw new Error(`${label} must use the exact fail-closed command body`);
  }
}

function rejectIgnoredFailures(value: UnknownRecord, label: string): void {
  const setting = value["continue-on-error"];
  if (setting !== undefined && setting !== false) {
    throw new Error(`${label} must not ignore failures with continue-on-error`);
  }
}

function rejectGoldenModeEnvironment(value: unknown, label: string): void {
  if (value === undefined) {
    return;
  }
  const environment = record(value, label);
  for (const key of FORBIDDEN_GOLDEN_MODES) {
    if (key in environment) {
      throw new Error(`${label} must not declare verification-bypassing ${key}`);
    }
  }
}

function rejectInheritedEnvironment(value: unknown, label: string): void {
  if (value !== undefined) {
    throw new Error(`${label} must be absent to isolate parity command execution`);
  }
}

function requireExactEnvironment(
  value: unknown,
  expected: UnknownRecord,
  label: string
): void {
  const environment = record(value, label);
  rejectGoldenModeEnvironment(environment, label);
  try {
    requireExactKeys(environment, Object.keys(expected), label);
  } catch {
    throw new Error(`${label} must contain exactly the approved environment keys`);
  }
  for (const [key, expectedValue] of Object.entries(expected)) {
    if (String(environment[key]) !== expectedValue) {
      throw new Error(`${label} ${key} must equal ${JSON.stringify(expectedValue)}`);
    }
  }
}

function rejectRunDefaults(value: unknown, label: string): void {
  if (value === undefined) {
    return;
  }
  const defaults = record(value, label);
  if (defaults.run !== undefined) {
    throw new Error(`${label}.run must not alter parity command execution`);
  }
}

function rejectLeafContainer(job: UnknownRecord, label: string): void {
  if (job.container !== undefined) {
    throw new Error(`${label}.container must not alter the reviewed runner environment`);
  }
}

function validateCriticalStep(step: UnknownRecord, expected: string, label: string): void {
  rejectIgnoredFailures(step, label);
  if (step.if !== undefined) {
    throw new Error(`${label} must not be conditionally skipped`);
  }
  requireExactCommand(step, expected, label);
}

function requireWorkingDirectory(
  step: UnknownRecord,
  expected: string | undefined,
  label: string
): void {
  if (step["working-directory"] !== expected) {
    throw new Error(`${label} working-directory must equal ${JSON.stringify(expected)}`);
  }
}

function validatePinnedToolchain(jobSteps: UnknownRecord[], label: string): number {
  const index = jobSteps.findIndex((step) => step.uses === "moonrepo/setup-toolchain@v0");
  if (index === -1) {
    throw new Error(`${label} must configure the pinned Moon toolchain`);
  }
  const step = jobSteps[index]!;
  rejectIgnoredFailures(step, `${label} toolchain step`);
  if (step.if !== undefined) {
    throw new Error(`${label} toolchain step must not be conditionally skipped`);
  }
  requireExactKeys(step, ["name", "uses", "with"], `${label} toolchain step`);
  const options = record(step.with, `${label} toolchain with`);
  requireExactKeys(options, ["auto-install", "auto-setup"], `${label} toolchain with`);
  if (options["auto-install"] !== true || options["auto-setup"] !== true) {
    throw new Error(`${label} must configure the pinned Moon toolchain`);
  }
  return index;
}

function validateCheckout(step: UnknownRecord, label: string): void {
  rejectIgnoredFailures(step, label);
  if (step.if !== undefined || step.uses !== "actions/checkout@v4") {
    throw new Error(`${label} must use the unconditional pinned checkout action`);
  }
  requireExactKeys(step, ["uses"], label);
}

function validateBootstrapBun(step: UnknownRecord): void {
  rejectIgnoredFailures(step, "openspec bootstrap Bun step");
  if (step.if !== undefined || step.uses !== "oven-sh/setup-bun@v2") {
    throw new Error("openspec bootstrap Bun step must use the unconditional pinned setup action");
  }
  requireExactKeys(step, ["name", "uses", "with"], "openspec bootstrap Bun step");
  const options = record(step.with, "openspec bootstrap Bun with");
  requireExactKeys(options, ["bun-version"], "openspec bootstrap Bun with");
  if (options["bun-version"] !== "1.4.0") {
    throw new Error('openspec bootstrap Bun version must equal "1.4.0"');
  }
}

function validateBootstrapMoon(step: UnknownRecord): void {
  rejectIgnoredFailures(step, "openspec bootstrap Moon step");
  if (step.if !== undefined || step.uses !== "moonrepo/setup-toolchain@v0") {
    throw new Error("openspec bootstrap Moon step must use the unconditional pinned setup action");
  }
  requireExactKeys(step, ["name", "uses", "with"], "openspec bootstrap Moon step");
  const options = record(step.with, "openspec bootstrap Moon with");
  requireExactKeys(
    options,
    ["auto-install", "auto-setup", "cache", "moon-version"],
    "openspec bootstrap Moon with"
  );
  if (
    options["auto-install"] !== false ||
    options["auto-setup"] !== false ||
    options.cache !== false ||
    options["moon-version"] !== "2.3.3"
  ) {
    throw new Error("openspec bootstrap Moon must use the exact isolated setup options");
  }
}

function validateOpenSpecJob(job: UnknownRecord): void {
  if (job.name !== "OpenSpec validation") {
    throw new Error("jobs.openspec.name must be OpenSpec validation");
  }
  if (job["runs-on"] !== "ubuntu-latest") {
    throw new Error("jobs.openspec.runs-on must be ubuntu-latest");
  }
  rejectIgnoredFailures(job, "jobs.openspec");
  rejectInheritedEnvironment(job.env, "jobs.openspec.env");
  rejectRunDefaults(job.defaults, "jobs.openspec.defaults");
  rejectLeafContainer(job, "jobs.openspec");
  if (job.if !== undefined) {
    throw new Error("jobs.openspec must not be conditionally skipped");
  }
  requireExactKeys(job, ["name", "outputs", "runs-on", "steps"], "jobs.openspec");
  const outputs = record(job.outputs, "jobs.openspec.outputs");
  requireExactKeys(outputs, ["policy_validated"], "jobs.openspec.outputs");
  if (outputs.policy_validated !== "${{ steps.policy.outputs.validated }}") {
    throw new Error("jobs.openspec policy_validated must expose the policy step attestation");
  }

  const jobSteps = steps(job, "jobs.openspec");
  requireExactStepSequence(
    jobSteps,
    [
      "Setup bootstrap Bun",
      "Setup bootstrap Moon",
      undefined,
      "Validate living specs and active changes",
    ],
    "openspec"
  );

  validateBootstrapBun(jobSteps[0]!);
  validateBootstrapMoon(jobSteps[1]!);
  const checkout = jobSteps[2]!;
  rejectIgnoredFailures(checkout, "openspec checkout step");
  if (checkout.if !== undefined || checkout.uses !== "actions/checkout@v4") {
    throw new Error("openspec checkout step must use the unconditional pinned checkout action");
  }
  requireExactKeys(checkout, ["uses", "with"], "openspec checkout step");
  const checkoutOptions = record(checkout.with, "openspec checkout with");
  requireExactKeys(checkoutOptions, ["fetch-depth"], "openspec checkout with");
  if (checkoutOptions["fetch-depth"] !== 0) {
    throw new Error("openspec checkout fetch-depth must equal 0");
  }

  const validation = requiredStep(
    jobSteps,
    "Validate living specs and active changes",
    "openspec"
  );
  validateCriticalStep(validation.step, OPENSPEC_COMMAND, "openspec validation step");
  requireWorkingDirectory(validation.step, undefined, "openspec validation step");
  requireExactKeys(validation.step, ["id", "name", "run"], "openspec validation step");
  if (validation.step.id !== "policy") {
    throw new Error('openspec validation step id must equal "policy"');
  }
}

export function validateOpenSpecTask(source: string): void {
  const config = record(Bun.YAML.parse(source), "moon config");
  requireExactKeys(config, ["$schema", "tasks", "workspace"], "root moon config");
  if (config.$schema !== "https://moonrepo.dev/schemas/project.json") {
    throw new Error("root moon config must use the pinned project schema");
  }
  const projectWorkspace = record(config.workspace, "root moon config workspace");
  requireExactKeys(projectWorkspace, ["inheritedTasks"], "root moon config workspace");
  const inheritedTasks = record(
    projectWorkspace.inheritedTasks,
    "root moon config inheritedTasks"
  );
  requireExactKeys(inheritedTasks, ["include"], "root moon config inheritedTasks");
  requireExactStringArray(
    inheritedTasks.include,
    [],
    "root moon config inheritedTasks.include"
  );
  const tasks = record(config.tasks, "moon config tasks");
  if (!("openspec-validate" in tasks)) {
    throw new Error("moon config must define tasks.openspec-validate");
  }
  const task = record(tasks["openspec-validate"], "tasks.openspec-validate");
  requireExactKeys(task, ["inputs", "options", "script"], "openspec-validate task");
  if (normalizedCommand(task.script) !== OPENSPEC_TASK_COMMAND) {
    throw new Error("openspec-validate task must use the exact fail-closed command sequence");
  }
  requireExactStringArray(task.inputs, OPENSPEC_TASK_INPUTS, "openspec-validate task inputs");
  const options = record(task.options, "openspec-validate task options");
  requireExactKeys(options, ["cache"], "openspec-validate task options");
  if (options.cache !== false) {
    throw new Error("openspec-validate task cache must remain disabled");
  }
}

function validateMoonWorkspace(source: string): void {
  const config = record(Bun.YAML.parse(source), "moon workspace config");
  requireExactKeys(
    config,
    ["$schema", "defaultProject", "projects", "vcs"],
    "moon workspace config"
  );
  if (config.$schema !== "https://moonrepo.dev/schemas/workspace.json") {
    throw new Error("moon workspace config must use the pinned workspace schema");
  }
  if (config.defaultProject !== "root") {
    throw new Error('moon workspace defaultProject must equal "root"');
  }
  const vcs = record(config.vcs, "moon workspace vcs");
  requireExactKeys(vcs, ["defaultBranch"], "moon workspace vcs");
  if (vcs.defaultBranch !== "main") {
    throw new Error('moon workspace default branch must equal "main"');
  }
  const projects = record(config.projects, "moon workspace projects");
  requireExactKeys(projects, ["globs", "sources"], "moon workspace projects");
  requireExactStringArray(
    projects.globs,
    ["apps/*", "crates/*"],
    "moon workspace project globs"
  );
  const sources = record(projects.sources, "moon workspace project sources");
  requireExactKeys(sources, ["root"], "moon workspace project sources");
  if (sources.root !== ".") {
    throw new Error('moon workspace root source must equal "."');
  }
}

function validateMoonToolchains(source: string): void {
  const config = record(Bun.YAML.parse(source), "moon toolchains config");
  requireExactKeys(
    config,
    ["$schema", "bun", "javascript", "rust"],
    "moon toolchains config"
  );
  if (config.$schema !== "https://moonrepo.dev/schemas/toolchains.json") {
    throw new Error("moon toolchains config must use the pinned toolchains schema");
  }
  const javascript = record(config.javascript, "moon javascript toolchain");
  requireExactKeys(javascript, ["packageManager"], "moon javascript toolchain");
  if (javascript.packageManager !== "bun") {
    throw new Error('moon javascript package manager must equal "bun"');
  }
  const bun = record(config.bun, "moon bun toolchain");
  requireExactKeys(bun, ["version"], "moon bun toolchain");
  if (bun.version !== "1.4.0") {
    throw new Error('moon bun version must equal "1.4.0"');
  }
  const rust = record(config.rust, "moon rust toolchain");
  requireExactKeys(rust, [], "moon rust toolchain");
}

function validateProtoConfiguration(source: string): void {
  if (normalizedCommand(source) !== PROTO_CONFIG) {
    throw new Error(".prototools must contain exactly the approved tool versions");
  }
}

function validateBunConfiguration(source: string): void {
  if (normalizedCommand(source) !== BUN_CONFIG) {
    throw new Error("bunfig.toml must contain exactly the approved Bun configuration");
  }
}

function validateGlobalTaskConfigurations(
  configurations: readonly MoonConfigurationSource[]
): void {
  const seen = new Set<string>();
  for (const configuration of configurations) {
    const path = configuration.path.replaceAll("\\", "/");
    if (!path.startsWith(`${DEFAULT_MOON_TASKS_DIRECTORY}/`)) {
      throw new Error(`global Moon task config path is outside ${DEFAULT_MOON_TASKS_DIRECTORY}`);
    }
    if (seen.has(path)) {
      throw new Error(`global Moon task config path is duplicated: ${path}`);
    }
    seen.add(path);
    if (![".yml", ".yaml"].includes(extname(path).toLowerCase())) {
      throw new Error(`unsupported global Moon task config must not affect policy execution: ${path}`);
    }
    const config = record(Bun.YAML.parse(configuration.source), `global Moon task config ${path}`);
    if ("env" in config) {
      throw new Error(`global Moon task config ${path} must not declare inherited env`);
    }
    if ("extends" in config) {
      throw new Error(`global Moon task config ${path} must not extend another config`);
    }
    requireAllowedKeys(config, ["$schema", "tasks"], `global Moon task config ${path}`);
  }
}

export function validateMoonPolicyBoundary(sources: MoonPolicySources): void {
  if (sources.activeChanges.length > 0) {
    throw new Error(
      `unarchived OpenSpec changes block merge readiness: ${[...sources.activeChanges]
        .sort()
        .join(", ")}`
    );
  }
  if (sources.unexpectedConfigurations.length > 0) {
    throw new Error(
      `unexpected Moon configuration may alter policy execution: ${[
        ...sources.unexpectedConfigurations,
      ]
        .sort()
        .join(", ")}`
    );
  }
  validateBunConfiguration(sources.bun);
  validateOpenSpecTask(sources.project);
  validateMoonWorkspace(sources.workspace);
  validateMoonToolchains(sources.toolchains);
  validateProtoConfiguration(sources.proto);
  validateGlobalTaskConfigurations(sources.globalTasks);
}

export function discoverActiveOpenSpecChanges(
  changesDirectory = DEFAULT_OPENSPEC_CHANGES_DIRECTORY
): string[] {
  if (!existsSync(changesDirectory)) {
    throw new Error(`OpenSpec changes root is missing: ${changesDirectory}`);
  }
  const changesMetadata = lstatSync(changesDirectory);
  if (!changesMetadata.isDirectory() || changesMetadata.isSymbolicLink()) {
    throw new Error(`OpenSpec changes root must be an ordinary directory: ${changesDirectory}`);
  }
  const archivePath = join(changesDirectory, OPENSPEC_ARCHIVE_DIRECTORY);
  if (!existsSync(archivePath)) {
    throw new Error(`OpenSpec archive directory is missing: ${archivePath}`);
  }
  const archiveMetadata = lstatSync(archivePath);
  if (!archiveMetadata.isDirectory() || archiveMetadata.isSymbolicLink()) {
    throw new Error(`OpenSpec archive must be an ordinary directory: ${archivePath}`);
  }
  return readdirSync(changesDirectory, { withFileTypes: true })
    .filter((entry) => entry.name !== OPENSPEC_ARCHIVE_DIRECTORY)
    .map((entry) => entry.name)
    .sort((left, right) => left.localeCompare(right));
}

function validateContractJob(job: UnknownRecord): void {
  if (job.name !== "Contract parity") {
    throw new Error("jobs.contract-parity.name must be Contract parity");
  }
  if (job["runs-on"] !== "ubuntu-latest") {
    throw new Error("jobs.contract-parity.runs-on must be ubuntu-latest");
  }
  rejectIgnoredFailures(job, "jobs.contract-parity");
  rejectInheritedEnvironment(job.env, "jobs.contract-parity.env");
  rejectRunDefaults(job.defaults, "jobs.contract-parity.defaults");
  rejectLeafContainer(job, "jobs.contract-parity");
  const jobSteps = steps(job, "jobs.contract-parity");
  requireExactStepSequence(
    jobSteps,
    [
      undefined,
      "Setup pinned toolchain",
      "Install JavaScript dependencies",
      "Cross-language contract parity",
    ],
    "contract-parity"
  );
  validateCheckout(jobSteps[0]!, "contract-parity checkout step");
  const toolchainIndex = validatePinnedToolchain(jobSteps, "contract-parity");
  const install = requiredStep(jobSteps, "Install JavaScript dependencies", "contract-parity");
  const parity = requiredStep(jobSteps, "Cross-language contract parity", "contract-parity");
  validateCriticalStep(install.step, "bun install --frozen-lockfile", "contract-parity install step");
  validateCriticalStep(parity.step, "bun run contracts:check", "contract-parity command step");
  requireWorkingDirectory(install.step, AGENT_BRIDGE_DIRECTORY, "contract-parity install step");
  requireWorkingDirectory(parity.step, AGENT_BRIDGE_DIRECTORY, "contract-parity command step");
  requireExactKeys(
    install.step,
    ["name", "run", "working-directory"],
    "contract-parity install step"
  );
  requireExactKeys(
    parity.step,
    ["name", "run", "working-directory"],
    "contract-parity command step"
  );
  if (!(toolchainIndex < install.index && install.index < parity.index)) {
    throw new Error("contract-parity setup and command steps must remain ordered");
  }
}

function validateRenderJob(job: UnknownRecord): void {
  if (job.name !== "Render parity") {
    throw new Error("jobs.render-parity.name must be Render parity");
  }
  if (job["runs-on"] !== "ubuntu-latest") {
    throw new Error("jobs.render-parity.runs-on must be ubuntu-latest");
  }
  rejectIgnoredFailures(job, "jobs.render-parity");
  rejectGoldenModeEnvironment(job.env, "jobs.render-parity.env");
  rejectInheritedEnvironment(job.env, "jobs.render-parity.env");
  rejectRunDefaults(job.defaults, "jobs.render-parity.defaults");
  rejectLeafContainer(job, "jobs.render-parity");
  const jobSteps = steps(job, "jobs.render-parity");
  requireExactStepSequence(
    jobSteps,
    [
      undefined,
      "Install deterministic rendering dependencies",
      "Setup pinned toolchain",
      "Native audiovisual and lifecycle parity",
      "Validate Linux render baseline schema",
      "Upload report-only Linux render baseline",
    ],
    "render-parity"
  );
  validateCheckout(jobSteps[0]!, "render-parity checkout step");
  const dependencies = requiredStep(
    jobSteps,
    "Install deterministic rendering dependencies",
    "render-parity"
  );
  const toolchainIndex = validatePinnedToolchain(jobSteps, "render-parity");
  const native = requiredStep(jobSteps, "Native audiovisual and lifecycle parity", "render-parity");
  const validation = requiredStep(jobSteps, "Validate Linux render baseline schema", "render-parity");
  const upload = requiredStep(jobSteps, "Upload report-only Linux render baseline", "render-parity");

  validateCriticalStep(
    dependencies.step,
    "sudo apt-get update && sudo apt-get install -y ffmpeg fonts-dejavu-core",
    "render-parity dependency step"
  );
  validateCriticalStep(native.step, NATIVE_PARITY_COMMAND, "render-parity native step");
  validateCriticalStep(
    validation.step,
    "cargo test -p opencut-editor-core renderer::golden::validate_external_performance_report -- --ignored --exact",
    "render-parity report-validation step"
  );
  rejectIgnoredFailures(upload.step, "render-parity upload step");
  if (upload.step.if !== undefined || upload.step.uses !== "actions/upload-artifact@v4") {
    throw new Error("render-parity upload must be the unconditional pinned artifact step");
  }

  for (const [step, label] of [
    [dependencies.step, "render-parity dependency step"],
    [native.step, "render-parity native step"],
    [validation.step, "render-parity report-validation step"],
  ] as const) {
    requireWorkingDirectory(step, undefined, label);
  }
  requireExactKeys(
    dependencies.step,
    ["name", "run"],
    "render-parity dependency step"
  );
  requireExactKeys(native.step, ["env", "name", "run"], "render-parity native step");
  requireExactKeys(
    validation.step,
    ["env", "name", "run"],
    "render-parity report-validation step"
  );

  const expectedEnvironment: UnknownRecord = {
    OPENCUT_FFMPEG_PATH: "ffmpeg",
    OPENCUT_FFPROBE_PATH: "ffprobe",
    OPENCUT_GOLDEN_REPORT_PATH: ABSOLUTE_REPORT_PATH,
    OPENCUT_GOLDEN_REQUIRED: "1",
    OPENCUT_TEST_FONT_PATH: "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
  };
  requireExactEnvironment(
    native.step.env,
    expectedEnvironment,
    "render-parity native env"
  );
  requireExactEnvironment(
    validation.step.env,
    { OPENCUT_GOLDEN_REPORT_PATH: ABSOLUTE_REPORT_PATH },
    "render-parity report-validation env"
  );
  const uploadOptions = record(upload.step.with, "render-parity upload with");
  requireExactKeys(upload.step, ["name", "uses", "with"], "render-parity upload step");
  requireExactKeys(uploadOptions, ["name", "path"], "render-parity upload with");
  if (uploadOptions.name !== "render-baseline-linux-v3") {
    throw new Error('render-parity upload name must equal "render-baseline-linux-v3"');
  }
  if (uploadOptions.path !== REPORT_PATH) {
    throw new Error(`render-parity upload path must equal ${JSON.stringify(REPORT_PATH)}`);
  }
  if (
    !(
      dependencies.index < toolchainIndex &&
      toolchainIndex < native.index &&
      native.index < validation.index &&
      validation.index < upload.index
    )
  ) {
    throw new Error("render-parity must validate the report before uploading it");
  }
}

export function assertFoundationParityResults(
  openspecResult: string,
  contractResult: string,
  renderResult: string,
  policyValidated: string
): void {
  if (
    openspecResult !== "success" ||
    contractResult !== "success" ||
    renderResult !== "success" ||
    policyValidated !== "true"
  ) {
    throw new Error(
      `foundation parity requires success results and policy attestation; openspec=${openspecResult}, contract=${contractResult}, render=${renderResult}, policy_validated=${policyValidated}`
    );
  }
}

function validateFoundationJob(job: UnknownRecord): void {
  if (job.name !== "Motion-graphics foundation parity") {
    throw new Error("jobs.foundation-parity.name must be Motion-graphics foundation parity");
  }
  if (job["runs-on"] !== "ubuntu-latest") {
    throw new Error("jobs.foundation-parity.runs-on must be ubuntu-latest");
  }
  rejectIgnoredFailures(job, "jobs.foundation-parity");
  rejectInheritedEnvironment(job.env, "jobs.foundation-parity.env");
  rejectRunDefaults(job.defaults, "jobs.foundation-parity.defaults");
  rejectLeafContainer(job, "jobs.foundation-parity");
  const needs = job.needs;
  if (
    !Array.isArray(needs) ||
    needs.length !== 3 ||
    !needs.includes("openspec") ||
    !needs.includes("contract-parity") ||
    !needs.includes("render-parity")
  ) {
    throw new Error(
      "jobs.foundation-parity.needs must contain exactly openspec, contract-parity, and render-parity"
    );
  }
  if (job.if !== FOUNDATION_CONDITION) {
    throw new Error(`jobs.foundation-parity.if must equal ${JSON.stringify(FOUNDATION_CONDITION)}`);
  }
  const jobSteps = steps(job, "jobs.foundation-parity");
  if (jobSteps.length !== 1) {
    throw new Error("jobs.foundation-parity must contain exactly one assertion step");
  }
  const assertion = requiredStep(jobSteps, "Confirm foundation parity gates", "foundation-parity");
  validateCriticalStep(assertion.step, FOUNDATION_COMMAND, "foundation-parity assertion step");
  requireExactKeys(
    assertion.step,
    ["name", "env", "run"],
    "foundation-parity assertion step"
  );
  const environment = record(assertion.step.env, "foundation-parity assertion env");
  try {
    requireExactKeys(
      environment,
      [
        "OPENSPEC_RESULT",
        "OPENSPEC_POLICY_VALIDATED",
        "CONTRACT_PARITY_RESULT",
        "RENDER_PARITY_RESULT",
      ],
      "foundation-parity assertion env"
    );
  } catch {
    throw new Error(
      "foundation-parity assertion env must contain exactly the approved environment keys"
    );
  }
  if (environment.OPENSPEC_RESULT !== "${{ needs.openspec.result }}") {
    throw new Error("foundation-parity must expose the openspec result");
  }
  if (
    environment.OPENSPEC_POLICY_VALIDATED !==
    "${{ needs.openspec.outputs.policy_validated }}"
  ) {
    throw new Error("foundation-parity must expose the openspec policy attestation");
  }
  if (environment.CONTRACT_PARITY_RESULT !== "${{ needs.contract-parity.result }}") {
    throw new Error("foundation-parity must expose the contract-parity result");
  }
  if (environment.RENDER_PARITY_RESULT !== "${{ needs.render-parity.result }}") {
    throw new Error("foundation-parity must expose the render-parity result");
  }
}

export function validateCiGates(source: string): void {
  const workflow = record(Bun.YAML.parse(source), "workflow");
  rejectGoldenModeEnvironment(workflow.env, "workflow.env");
  rejectInheritedEnvironment(workflow.env, "workflow.env");
  rejectRunDefaults(workflow.defaults, "workflow.defaults");
  const jobs = record(workflow.jobs, "workflow.jobs");
  validateOpenSpecJob(requiredJob(jobs, "openspec"));
  validateContractJob(requiredJob(jobs, "contract-parity"));
  validateRenderJob(requiredJob(jobs, "render-parity"));
  validateFoundationJob(requiredJob(jobs, "foundation-parity"));
}

export function validateCiPolicy(
  workflowSource: string,
  moonSources: MoonPolicySources
): void {
  validateMoonPolicyBoundary(moonSources);
  validateCiGates(workflowSource);
}

function discoverGlobalTaskConfigurations(
  directory = DEFAULT_MOON_TASKS_DIRECTORY
): MoonConfigurationSource[] {
  if (!existsSync(directory)) {
    return [];
  }
  if (lstatSync(directory).isSymbolicLink()) {
    throw new Error(`symbolic Moon configuration paths are not allowed: ${directory}`);
  }
  const paths: string[] = [];
  const visit = (current: string): void => {
    for (const entry of readdirSync(current, { withFileTypes: true })) {
      const path = join(current, entry.name);
      if (entry.isSymbolicLink()) {
        throw new Error(`symbolic Moon configuration paths are not allowed: ${path}`);
      }
      if (entry.isDirectory()) {
        visit(path);
      } else if (entry.isFile()) {
        paths.push(path);
      } else {
        throw new Error(`unsupported Moon configuration path: ${path}`);
      }
    }
  };
  visit(directory);
  return paths
    .sort((left, right) => left.localeCompare(right))
    .map((path) => ({ path, source: readFileSync(path, "utf8") }));
}

function discoverUnexpectedMoonConfigurations(): string[] {
  const unexpected: string[] = [];
  const rootNames = readdirSync(".", { withFileTypes: true });
  for (const entry of rootNames) {
    if (
      (entry.isFile() || entry.isSymbolicLink()) &&
      (entry.name.startsWith(".prototools.") || entry.name === "prototools")
    ) {
      unexpected.push(entry.name);
    }
    if (
      (entry.isFile() || entry.isSymbolicLink()) &&
      entry.name.startsWith("moon.") &&
      entry.name !== DEFAULT_MOON_CONFIG &&
      MOON_CONFIG_EXTENSIONS.includes(extname(entry.name).toLowerCase())
    ) {
      unexpected.push(entry.name);
    }
  }
  for (const entry of readdirSync(".moon", { withFileTypes: true })) {
    if (!entry.isFile() && !entry.isSymbolicLink()) {
      continue;
    }
    const extension = extname(entry.name).toLowerCase();
    if (!MOON_CONFIG_EXTENSIONS.includes(extension)) {
      continue;
    }
    const stem = entry.name.slice(0, -extension.length);
    if (
      (stem === "workspace" && entry.name !== "workspace.yml") ||
      (stem === "toolchains" && entry.name !== "toolchains.yml") ||
      stem === "extensions" ||
      stem === "tasks"
    ) {
      unexpected.push(`.moon/${entry.name}`);
    }
  }
  return unexpected.sort((left, right) => left.localeCompare(right));
}

function readRequiredPolicyConfiguration(path: string): string {
  if (!existsSync(path)) {
    throw new Error(`required policy configuration is missing: ${path}`);
  }
  const metadata = lstatSync(path);
  if (!metadata.isFile() || metadata.isSymbolicLink()) {
    throw new Error(`required policy configuration must be a regular file: ${path}`);
  }
  return readFileSync(path, "utf8");
}

export function loadMoonPolicySources(): MoonPolicySources {
  return {
    bun: readRequiredPolicyConfiguration(DEFAULT_BUN_CONFIG),
    project: readRequiredPolicyConfiguration(DEFAULT_MOON_CONFIG),
    workspace: readRequiredPolicyConfiguration(DEFAULT_MOON_WORKSPACE_CONFIG),
    toolchains: readRequiredPolicyConfiguration(DEFAULT_MOON_TOOLCHAINS_CONFIG),
    proto: readRequiredPolicyConfiguration(DEFAULT_PROTO_CONFIG),
    globalTasks: discoverGlobalTaskConfigurations(),
    unexpectedConfigurations: discoverUnexpectedMoonConfigurations(),
    activeChanges: discoverActiveOpenSpecChanges(),
  };
}

function workflowArgument(args: string[]): string {
  const index = args.indexOf("--workflow");
  if (index === -1) {
    return DEFAULT_WORKFLOW;
  }
  const value = args[index + 1];
  if (!value) {
    throw new Error("--workflow requires a path");
  }
  return value;
}

if (import.meta.main) {
  const args = Bun.argv.slice(2);
  const workflowPath = workflowArgument(args);
  try {
    if (args.includes("--attest-github-output")) {
      throw new Error("validate-ci-gates is validation-only and must not emit attestations");
    }
    validateCiPolicy(await Bun.file(workflowPath).text(), loadMoonPolicySources());
    console.log(`CI parity gate policy is valid: ${workflowPath}`);
  } catch (error) {
    console.error(
      `CI parity gate policy failed: ${error instanceof Error ? error.message : String(error)}`
    );
    process.exitCode = 1;
  }
}

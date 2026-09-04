import { describe, expect, it } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import {
  assertFoundationParityResults,
  discoverActiveOpenSpecChanges,
  type MoonPolicySources,
  validateCiGates,
  validateCiPolicy,
  validateMoonPolicyBoundary,
  validateOpenSpecTask,
} from "./validate-ci-gates";

const workflow = readFileSync(
  resolve(import.meta.dir, "..", ".github", "workflows", "bun-ci.yml"),
  "utf8"
);
const moonConfig = readFileSync(resolve(import.meta.dir, "..", "moon.yml"), "utf8");
const moonWorkspaceConfig = readFileSync(
  resolve(import.meta.dir, "..", ".moon", "workspace.yml"),
  "utf8"
);
const moonToolchainsConfig = readFileSync(
  resolve(import.meta.dir, "..", ".moon", "toolchains.yml"),
  "utf8"
);
const protoConfig = readFileSync(resolve(import.meta.dir, "..", ".prototools"), "utf8");
const bunConfig = readFileSync(resolve(import.meta.dir, "..", "bunfig.toml"), "utf8");
const workflowBootstrapCommand =
  "bun --config=/dev/null --no-env-file run scripts/run-ci-policy.ts --attest-github-output";
const protectedBun = "bun --config=bunfig.toml --no-env-file";

function moonPolicySources(overrides: Partial<MoonPolicySources> = {}): MoonPolicySources {
  return {
    bun: bunConfig,
    project: moonConfig,
    workspace: moonWorkspaceConfig,
    toolchains: moonToolchainsConfig,
    proto: protoConfig,
    globalTasks: [],
    unexpectedConfigurations: [],
    activeChanges: [],
    ...overrides,
  };
}

function replaceRequired(source: string, needle: string, replacement: string): string {
  expect(source).toContain(needle);
  return source.replace(needle, replacement);
}

function addContinueOnError(source: string, stepName: string): string {
  const declaration = `      - name: ${stepName}\n`;
  return replaceRequired(
    source,
    declaration,
    `${declaration}        continue-on-error: true\n`
  );
}

function addStepProperty(source: string, stepName: string, property: string): string {
  const declaration = `      - name: ${stepName}\n`;
  return replaceRequired(source, declaration, `${declaration}        ${property}\n`);
}

function insertStepBefore(source: string, stepName: string, step: string): string {
  const declaration = `      - name: ${stepName}\n`;
  return replaceRequired(source, declaration, `${step}${declaration}`);
}

function addJobConfiguration(source: string, jobId: string, configuration: string): string {
  const declaration = `  ${jobId}:\n`;
  return replaceRequired(source, declaration, `${declaration}${configuration}`);
}

type EnvironmentScope = "workflow" | "job" | "native" | "validation";

function addEnvironmentVariable(
  source: string,
  scope: EnvironmentScope,
  key: string,
  value: string
): string {
  if (scope === "workflow") {
    return replaceRequired(
      source,
      "name: OpenCut CI\n",
      `name: OpenCut CI\nenv:\n  ${key}: ${value}\n`
    );
  }
  if (scope === "job") {
    return replaceRequired(
      source,
      "  render-parity:\n",
      `  render-parity:\n    env:\n      ${key}: ${value}\n`
    );
  }
  const stepName =
    scope === "native"
      ? "Native audiovisual and lifecycle parity"
      : "Validate Linux render baseline schema";
  const declaration = `      - name: ${stepName}\n        env:\n`;
  return replaceRequired(
    source,
    declaration,
    `${declaration}          ${key}: ${value}\n`
  );
}

function moveUploadBeforeValidation(source: string): string {
  const validationStart = source.indexOf("      - name: Validate Linux render baseline schema\n");
  const uploadStart = source.indexOf("      - name: Upload report-only Linux render baseline\n");
  const foundationStart = source.indexOf("\n  foundation-parity:", uploadStart);
  expect(validationStart).toBeGreaterThan(-1);
  expect(uploadStart).toBeGreaterThan(validationStart);
  expect(foundationStart).toBeGreaterThan(uploadStart);
  const validationBlock = source.slice(validationStart, uploadStart);
  const uploadBlock = source.slice(uploadStart, foundationStart);
  return (
    source.slice(0, validationStart) +
    uploadBlock +
    validationBlock +
    source.slice(foundationStart)
  );
}

function moveOpenSpecValidationBeforeToolchain(source: string): string {
  const toolchainStart = source.indexOf("      - name: Setup bootstrap Moon\n");
  const validationStart = source.indexOf("      - name: Validate living specs and active changes\n");
  const contractStart = source.indexOf("\n  contract-parity:", validationStart);
  expect(toolchainStart).toBeGreaterThan(-1);
  expect(validationStart).toBeGreaterThan(toolchainStart);
  expect(contractStart).toBeGreaterThan(validationStart);
  const toolchainBlock = source.slice(toolchainStart, validationStart);
  const validationBlock = source.slice(validationStart, contractStart);
  return (
    source.slice(0, toolchainStart) +
    validationBlock +
    toolchainBlock +
    source.slice(contractStart)
  );
}

describe("foundation parity result assertion", () => {
  it("accepts only three successful prerequisite results with a true policy attestation", () => {
    const results = ["success", "failure", "cancelled", "skipped"];
    const attestations = ["true", "", "false", "unexpected"];
    for (const openspecResult of results) {
      for (const contractResult of results) {
        for (const renderResult of results) {
          for (const policyValidated of attestations) {
            if (
              openspecResult === "success" &&
              contractResult === "success" &&
              renderResult === "success" &&
              policyValidated === "true"
            ) {
              expect(() =>
                assertFoundationParityResults(
                  openspecResult,
                  contractResult,
                  renderResult,
                  policyValidated
                )
              ).not.toThrow();
            } else {
              expect(() =>
                assertFoundationParityResults(
                  openspecResult,
                  contractResult,
                  renderResult,
                  policyValidated
                )
              ).toThrow("foundation parity requires success results and policy attestation");
            }
          }
        }
      }
    }
  });

  it("rejects a failed policy result even when both parity leaves succeed", () => {
    expect(() =>
      assertFoundationParityResults("failure", "success", "success", "true")
    ).toThrow("openspec=failure, contract=success, render=success");
  });

  it("rejects a masked policy failure when the job result is success but attestation is absent", () => {
    expect(() =>
      assertFoundationParityResults("success", "success", "success", "")
    ).toThrow("policy_validated=");
  });
});

describe("OpenSpec task policy attestation", () => {
  it("accepts the checked-in Moon task", () => {
    expect(() => validateOpenSpecTask(moonConfig)).not.toThrow();
  });

  it("validates both structural inputs without owning an output writer", () => {
    expect(() => validateCiPolicy(workflow, moonPolicySources())).not.toThrow();
  });

  it("rejects the retired attestation flag", () => {
    const directory = mkdtempSync(join(tmpdir(), "opencut-policy-attestation-"));
    const outputPath = join(directory, "github-output");
    writeFileSync(outputPath, "existing=value\n", "utf8");
    try {
      const result = Bun.spawnSync({
        cmd: [
          process.execPath,
          "run",
          resolve(import.meta.dir, "validate-ci-gates.ts"),
          "--attest-github-output",
        ],
        cwd: resolve(import.meta.dir, ".."),
        env: { ...process.env, GITHUB_OUTPUT: outputPath },
        stdout: "pipe",
        stderr: "pipe",
      });
      expect(result.exitCode).toBe(1);
      expect(new TextDecoder().decode(result.stderr)).toContain("validation-only");
      expect(readFileSync(outputPath, "utf8")).toBe("existing=value\n");
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  });

  it("rejects a workflow bootstrap command masked with || true", () => {
    const mutation = replaceRequired(
      workflow,
      `run: ${workflowBootstrapCommand}`,
      `run: ${workflowBootstrapCommand} || true`
    );
    expect(() => validateCiPolicy(mutation, moonPolicySources())).toThrow(
      "openspec validation step must use the exact fail-closed command body"
    );
  });

  it("rejects an altered Moon task", () => {
    const mutation = replaceRequired(
      moonConfig,
      `${protectedBun} run scripts/validate-ci-gates.ts`,
      `${protectedBun} run scripts/validate-ci-gates.ts || true`
    );
    expect(() => validateCiPolicy(workflow, moonPolicySources({ project: mutation }))).toThrow(
      "openspec-validate task must use the exact fail-closed command sequence"
    );
  });

  for (const [description, mutation] of [
    [
      "missing final validator command",
      replaceRequired(
        moonConfig,
        `      ${protectedBun} run scripts/validate-ci-gates.ts\n`,
        ""
      ),
    ],
    [
      "duplicated final validator command",
      replaceRequired(
        moonConfig,
        `      ${protectedBun} run scripts/validate-ci-gates.ts\n`,
        `      ${protectedBun} run scripts/validate-ci-gates.ts\n      ${protectedBun} run scripts/validate-ci-gates.ts\n`
      ),
    ],
    [
      "premature validator command",
      replaceRequired(
        moonConfig,
        `      ${protectedBun} x @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive &&\n      ${protectedBun} run scripts/validate-ci-gates.ts\n`,
        `      ${protectedBun} run scripts/validate-ci-gates.ts &&\n      ${protectedBun} x @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive\n`
      ),
    ],
    [
      "non-fail-closed Moon command boundary",
      replaceRequired(
        moonConfig,
        `      ${protectedBun} run scripts/normalize-openspec-workflows.ts --check &&\n`,
        `      ${protectedBun} run scripts/normalize-openspec-workflows.ts --check\n`
      ),
    ],
    [
      "inline Moon output write",
      replaceRequired(
        moonConfig,
        `      ${protectedBun} run scripts/validate-ci-gates.ts\n`,
        `      echo "validated=true" >> "$GITHUB_OUTPUT"\n      ${protectedBun} run scripts/validate-ci-gates.ts\n`
      ),
    ],
  ] as const) {
    it(`rejects ${description}`, () => {
      expect(() => validateOpenSpecTask(mutation)).toThrow(
        "openspec-validate task must use the exact fail-closed command sequence"
      );
    });
  }

  it("rejects additional Moon task properties", () => {
    expect(() =>
      validateOpenSpecTask(
        replaceRequired(
          moonConfig,
          "  openspec-validate:\n",
          "  openspec-validate:\n    env:\n      BASH_ENV: injected\n"
        )
      )
    ).toThrow("openspec-validate task must contain exactly the approved properties");
  });

  for (const [name, value] of [
    ["empty", "{}"],
    ["BASH_ENV", "scripts/forge-policy.sh"],
    ["PATH", "${PATH}:scripts/fake-bin"],
    ["LD_PRELOAD", "scripts/forge.so"],
    ["LD_AUDIT", "scripts/audit.so"],
    ["NODE_OPTIONS", "--require scripts/forge.cjs"],
    ["CI_LOG_LEVEL", "debug"],
  ] as const) {
    it(`rejects root Moon environment ${name}`, () => {
      const environment =
        name === "empty" ? "env: {}\n" : `env:\n  ${name}: '${value}'\n`;
      expect(() =>
        validateOpenSpecTask(
          replaceRequired(
            moonConfig,
            "$schema: 'https://moonrepo.dev/schemas/project.json'\n",
            `$schema: 'https://moonrepo.dev/schemas/project.json'\n${environment}`
          )
        )
      ).toThrow("root moon config must contain exactly the approved properties");
    });
  }

  for (const override of ["platform: bun", "toolchain: system", "docker: {}"] as const) {
    it(`rejects root Moon execution override ${override}`, () => {
      expect(() =>
        validateOpenSpecTask(
          replaceRequired(
            moonConfig,
            "$schema: 'https://moonrepo.dev/schemas/project.json'\n",
            `$schema: 'https://moonrepo.dev/schemas/project.json'\n${override}\n`
          )
        )
      ).toThrow("root moon config must contain exactly the approved properties");
    });
  }

  it("rejects missing root task-inheritance isolation", () => {
    expect(() =>
      validateOpenSpecTask(
        replaceRequired(
          moonConfig,
          "workspace:\n  inheritedTasks:\n    include: []\n\n",
          ""
        )
      )
    ).toThrow("root moon config must contain exactly the approved properties");
  });

  it("rejects altered root task-inheritance isolation", () => {
    expect(() =>
      validateOpenSpecTask(replaceRequired(moonConfig, "include: []", "include: ['build']"))
    ).toThrow("root moon config inheritedTasks.include");
  });

  it("accepts the complete checked-in Moon execution boundary", () => {
    expect(() => validateMoonPolicyBoundary(moonPolicySources())).not.toThrow();
  });

  it("accepts an archive-only OpenSpec changes directory", () => {
    const directory = mkdtempSync(join(tmpdir(), "opencut-archived-changes-"));
    const changes = join(directory, "changes");
    mkdirSync(join(changes, "archive"), { recursive: true });
    try {
      expect(discoverActiveOpenSpecChanges(changes)).toEqual([]);
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  });

  it("rejects every unarchived OpenSpec entry independent of entry type or name", () => {
    const directory = mkdtempSync(join(tmpdir(), "opencut-active-changes-"));
    const changes = join(directory, "changes");
    mkdirSync(join(changes, "archive"), { recursive: true });
    mkdirSync(join(changes, "proposal-a"));
    writeFileSync(join(changes, "notes.txt"), "pending", "utf8");
    try {
      expect(discoverActiveOpenSpecChanges(changes)).toEqual(["notes.txt", "proposal-a"]);
      expect(() =>
        validateMoonPolicyBoundary(
          moonPolicySources({ activeChanges: ["proposal-a", "notes.txt"] })
        )
      ).toThrow(
        "unarchived OpenSpec changes block merge readiness: notes.txt, proposal-a"
      );
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  });

  it("rejects a symbolic-link entry as an unarchived OpenSpec change", () => {
    const directory = mkdtempSync(join(tmpdir(), "opencut-linked-change-"));
    const changes = join(directory, "changes");
    const target = join(directory, "target");
    mkdirSync(join(changes, "archive"), { recursive: true });
    mkdirSync(target);
    try {
      symlinkSync(target, join(changes, "linked-change"), "junction");
      expect(discoverActiveOpenSpecChanges(changes)).toEqual(["linked-change"]);
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  });

  for (const boundary of [
    "missing root",
    "file root",
    "missing archive",
    "file archive",
    "linked archive",
  ] as const) {
    it(`rejects an invalid OpenSpec archive boundary: ${boundary}`, () => {
      const directory = mkdtempSync(join(tmpdir(), "opencut-invalid-archive-"));
      const changes = join(directory, "changes");
      try {
        if (boundary === "file root") {
          writeFileSync(changes, "invalid", "utf8");
        } else if (boundary !== "missing root") {
          mkdirSync(changes);
        }
        if (boundary === "file archive") {
          writeFileSync(join(changes, "archive"), "invalid", "utf8");
        }
        if (boundary === "linked archive") {
          const target = join(directory, "archive-target");
          mkdirSync(target);
          symlinkSync(target, join(changes, "archive"), "junction");
        }
        expect(() => discoverActiveOpenSpecChanges(changes)).toThrow(
          boundary === "missing root"
            ? "OpenSpec changes root is missing"
            : boundary === "file root"
              ? "OpenSpec changes root must be an ordinary directory"
            : boundary === "missing archive"
              ? "OpenSpec archive directory is missing"
              : "OpenSpec archive must be an ordinary directory"
        );
      } finally {
        rmSync(directory, { recursive: true, force: true });
      }
    });
  }

  for (const [description, source] of [
    ["missing content", ""],
    ["changed install policy", bunConfig.replace("604800", "0")],
    ["preload", `${bunConfig}\npreload = [\"./scripts/forge-policy.ts\"]\n`],
    ["import", `${bunConfig}\n[run]\npreload = [\"./scripts/forge-policy.ts\"]\n`],
    ["additional setting", `${bunConfig}\nregistry = \"https://example.invalid\"\n`],
  ] as const) {
    it(`rejects canonical Bun configuration ${description}`, () => {
      expect(() =>
        validateMoonPolicyBoundary(moonPolicySources({ bun: source }))
      ).toThrow("bunfig.toml must contain exactly the approved Bun configuration");
    });
  }

  for (const [description, needle, replacement] of [
    [
      "a command without the reviewed Bun config",
      `${protectedBun} run scripts/normalize-openspec-workflows.ts --check`,
      "bun --no-env-file run scripts/normalize-openspec-workflows.ts --check",
    ],
    [
      "a command with automatic dotenv loading",
      `${protectedBun} run scripts/validate-ci-gates.ts`,
      "bun --config=bunfig.toml run scripts/validate-ci-gates.ts",
    ],
    [
      "a policy test command without the real-Moon regression",
      "scripts/run-ci-policy.test.ts scripts/run-ci-policy.integration.test.ts",
      "scripts/run-ci-policy.test.ts",
    ],
  ] as const) {
    it(`rejects ${description}`, () => {
      expect(() => validateOpenSpecTask(replaceRequired(moonConfig, needle, replacement))).toThrow(
        "openspec-validate task must use the exact fail-closed command sequence"
      );
    });
  }

  for (const [description, source, expected] of [
    [
      "redirected default project",
      replaceRequired(moonWorkspaceConfig, "defaultProject: 'root'", "defaultProject: 'web'"),
      'moon workspace defaultProject must equal "root"',
    ],
    [
      "redirected root source",
      replaceRequired(moonWorkspaceConfig, "root: '.'", "root: 'apps/web'"),
      'moon workspace root source must equal "."',
    ],
    [
      "workspace extension",
      replaceRequired(
        moonWorkspaceConfig,
        "$schema: 'https://moonrepo.dev/schemas/workspace.json'\n",
        "$schema: 'https://moonrepo.dev/schemas/workspace.json'\nextends: remote.yml\n"
      ),
      "moon workspace config must contain exactly the approved properties",
    ],
  ] as const) {
    it(`rejects ${description}`, () => {
      expect(() =>
        validateMoonPolicyBoundary(moonPolicySources({ workspace: source }))
      ).toThrow(expected);
    });
  }

  for (const [description, source, expected] of [
    [
      "changed Bun package manager",
      replaceRequired(moonToolchainsConfig, "packageManager: 'bun'", "packageManager: 'npm'"),
      'moon javascript package manager must equal "bun"',
    ],
    [
      "changed Bun version",
      replaceRequired(moonToolchainsConfig, "version: '1.4.0'", "version: '1.4.1'"),
      'moon bun version must equal "1.4.0"',
    ],
    [
      "toolchain extension",
      replaceRequired(
        moonToolchainsConfig,
        "$schema: 'https://moonrepo.dev/schemas/toolchains.json'\n",
        "$schema: 'https://moonrepo.dev/schemas/toolchains.json'\nextends: remote.yml\n"
      ),
      "moon toolchains config must contain exactly the approved properties",
    ],
  ] as const) {
    it(`rejects ${description}`, () => {
      expect(() =>
        validateMoonPolicyBoundary(moonPolicySources({ toolchains: source }))
      ).toThrow(expected);
    });
  }

  it("permits global tasks for non-root projects", () => {
    expect(() =>
      validateMoonPolicyBoundary(
        moonPolicySources({
          globalTasks: [
            {
              path: ".moon/tasks/web.yml",
              source:
                "$schema: 'https://moonrepo.dev/schemas/tasks.json'\ntasks:\n  build:\n    command: 'bun run build'\n",
            },
          ],
        })
      )
    ).not.toThrow();
  });

  for (const [description, source, expected] of [
    ["empty inherited env", "env: {}\ntasks: {}\n", "must not declare inherited env"],
    [
      "inherited BASH_ENV",
      "env:\n  BASH_ENV: scripts/forge-policy.sh\ntasks: {}\n",
      "must not declare inherited env",
    ],
    ["external extension", "extends: remote.yml\ntasks: {}\n", "must not extend another config"],
    ["implicit task options", "taskOptions:\n  allowFailure: true\ntasks: {}\n", "unapproved properties"],
  ] as const) {
    it(`rejects global Moon task ${description}`, () => {
      expect(() =>
        validateMoonPolicyBoundary(
          moonPolicySources({
            globalTasks: [{ path: ".moon/tasks/injected.yml", source }],
          })
        )
      ).toThrow(expected);
    });
  }

  it("rejects unsupported global Moon configuration files", () => {
    expect(() =>
      validateMoonPolicyBoundary(
        moonPolicySources({
          globalTasks: [{ path: ".moon/tasks/injected.json", source: "{}" }],
        })
      )
    ).toThrow("unsupported global Moon task config");
  });

  it("rejects an alternate root Moon configuration", () => {
    expect(() =>
      validateMoonPolicyBoundary(
        moonPolicySources({ unexpectedConfigurations: ["moon.json"] })
      )
    ).toThrow("unexpected Moon configuration may alter policy execution: moon.json");
  });

  it("rejects Moon pipeline extensions", () => {
    expect(() =>
      validateMoonPolicyBoundary(
        moonPolicySources({ unexpectedConfigurations: [".moon/extensions.yml"] })
      )
    ).toThrow("unexpected Moon configuration may alter policy execution");
  });

  for (const [description, source] of [
    ["changed Moon version", protoConfig.replace('moon = "2.3.3"', 'moon = "2.3.4"')],
    ["changed Bun version", protoConfig.replace('bun  = "1.4.0"', 'bun  = "1.4.1"')],
    ["changed Rust version", protoConfig.replace('rust = "1.97.0"', 'rust = "stable"')],
    ["environment injection", `${protoConfig}\n[env]\nBASH_ENV = "scripts/forge.sh"\n`],
    ["plugin injection", `${protoConfig}\n[plugins]\nmoon = "https://example.invalid/moon.wasm"\n`],
  ] as const) {
    it(`rejects .prototools ${description}`, () => {
      expect(() =>
        validateMoonPolicyBoundary(moonPolicySources({ proto: source }))
      ).toThrow(".prototools must contain exactly the approved tool versions");
    });
  }

  it("rejects an alternate proto configuration", () => {
    expect(() =>
      validateMoonPolicyBoundary(
        moonPolicySources({ unexpectedConfigurations: [".prototools.local"] })
      )
    ).toThrow("unexpected Moon configuration may alter policy execution");
  });

  it("rejects the reviewed project-level BASH_ENV bypass", () => {
    const mutation = replaceRequired(
      moonConfig,
      "$schema: 'https://moonrepo.dev/schemas/project.json'\n",
      "$schema: 'https://moonrepo.dev/schemas/project.json'\nenv:\n  BASH_ENV: scripts/forge-policy.sh\n"
    );
    expect(() =>
      validateCiPolicy(workflow, moonPolicySources({ project: mutation }))
    ).toThrow("root moon config must contain exactly the approved properties");
  });
});

describe("CI parity gate policy", () => {
  it("accepts the checked-in workflow", () => {
    expect(() => validateCiGates(workflow)).not.toThrow();
  });

  it("accepts environment configuration on a non-parity job", () => {
    expect(() =>
      validateCiGates(
        addJobConfiguration(workflow, "correctness", "    env:\n      CI_LOG_LEVEL: info\n")
      )
    ).not.toThrow();
  });

  for (const variable of [
    "BASH_ENV",
    "PATH",
    "LD_PRELOAD",
    "LD_AUDIT",
    "NODE_OPTIONS",
    "CI_LOG_LEVEL",
  ]) {
    for (const value of ["value", "${{ github.workspace }}/injected-value"]) {
      it(`rejects workflow ${variable} with value ${value}`, () => {
        expect(() =>
          validateCiGates(addEnvironmentVariable(workflow, "workflow", variable, value))
        ).toThrow("workflow.env must be absent to isolate parity command execution");
      });

      for (const jobId of [
        "openspec",
        "contract-parity",
        "render-parity",
        "foundation-parity",
      ]) {
        it(`rejects ${jobId} ${variable} with value ${value}`, () => {
          expect(() =>
            validateCiGates(
              addJobConfiguration(
                workflow,
                jobId,
                `    env:\n      ${variable}: ${value}\n`
              )
            )
          ).toThrow(`jobs.${jobId}.env must be absent to isolate parity command execution`);
        });
      }
    }
  }

  it("rejects an empty workflow environment map", () => {
    expect(() =>
      validateCiGates(replaceRequired(workflow, "name: OpenCut CI\n", "name: OpenCut CI\nenv: {}\n"))
    ).toThrow("workflow.env must be absent to isolate parity command execution");
  });

  for (const jobId of [
    "openspec",
    "contract-parity",
    "render-parity",
    "foundation-parity",
  ]) {
    it(`rejects an empty ${jobId} environment map`, () => {
      expect(() =>
        validateCiGates(addJobConfiguration(workflow, jobId, "    env: {}\n"))
      ).toThrow(`jobs.${jobId}.env must be absent to isolate parity command execution`);
    });
  }

  for (const [jobId, targetStep, injectedStep] of [
    [
      "contract-parity",
      "Cross-language contract parity",
      "      - name: Rewrite contract fixture\n        run: echo modified > contracts/fixture.json\n",
    ],
    [
      "render-parity",
      "Native audiovisual and lifecycle parity",
      "      - name: Rewrite golden fixture\n        run: echo modified > crates/editor-core/tests/fixtures/render-golden/CURRENT\n",
    ],
  ] as const) {
    it(`rejects an added repository-mutating step in ${jobId}`, () => {
      expect(() =>
        validateCiGates(insertStepBefore(workflow, targetStep, injectedStep))
      ).toThrow(`${jobId} must contain exactly its approved step sequence`);
    });
  }

  it("rejects an additional action in a leaf job", () => {
    expect(() =>
      validateCiGates(
        insertStepBefore(
          workflow,
          "Cross-language contract parity",
          "      - name: Unreviewed preparation action\n        uses: example/setup@v1\n"
        )
      )
    ).toThrow("contract-parity must contain exactly its approved step sequence");
  });

  for (const variable of [
    "OPENCUT_UPDATE_GOLDENS",
    "OPENCUT_CAPTURE_GOLDENS_TO",
  ]) {
    it(`rejects ${variable} persisted through GITHUB_ENV`, () => {
      expect(() =>
        validateCiGates(
          insertStepBefore(
            workflow,
            "Native audiovisual and lifecycle parity",
            `      - name: Persist golden mode\n        run: echo ${variable}=1 >> "$GITHUB_ENV"\n`
          )
        )
      ).toThrow("render-parity must contain exactly its approved step sequence");
    });
  }

  for (const [description, mutation, expected] of [
    [
      "duplicate contract step",
      insertStepBefore(
        workflow,
        "Cross-language contract parity",
        "      - name: Cross-language contract parity\n        working-directory: apps/agent-bridge\n        run: bun run contracts:check\n"
      ),
      "contract-parity must contain exactly its approved step sequence",
    ],
    [
      "missing render validation step",
      replaceRequired(
        workflow,
        "      - name: Validate Linux render baseline schema\n",
        "      - name: Removed render validation step\n"
      ),
      "render-parity.steps[4] must be",
    ],
    [
      "replaced contract install step",
      replaceRequired(
        workflow,
        "      - name: Install JavaScript dependencies\n",
        "      - name: Alternate dependency installation\n"
      ),
      "contract-parity.steps[2] must be",
    ],
    [
      "reordered render steps",
      moveUploadBeforeValidation(workflow),
      "render-parity.steps[4] must be",
    ],
  ] as const) {
    it(`rejects a ${description}`, () => {
      expect(() => validateCiGates(mutation)).toThrow(expected);
    });
  }

  for (const [jobId, stepName] of [
    ["contract-parity", "Cross-language contract parity"],
    ["render-parity", "Native audiovisual and lifecycle parity"],
  ] as const) {
    it(`rejects a custom shell in ${jobId}`, () => {
      expect(() =>
        validateCiGates(addStepProperty(workflow, stepName, "shell: bash"))
      ).toThrow("must contain exactly the approved properties");
    });

    it(`rejects command defaults in ${jobId}`, () => {
      expect(() =>
        validateCiGates(
          addJobConfiguration(
            workflow,
            jobId,
            "    defaults:\n      run:\n        shell: bash\n"
          )
        )
      ).toThrow(`jobs.${jobId}.defaults.run must not alter parity command execution`);
    });

    it(`rejects a container in ${jobId}`, () => {
      expect(() =>
        validateCiGates(
          addJobConfiguration(
            workflow,
            jobId,
            "    container:\n      image: ubuntu:latest\n      env:\n        OPENCUT_UPDATE_GOLDENS: 1\n"
          )
        )
      ).toThrow(`jobs.${jobId}.container must not alter the reviewed runner environment`);
    });
  }

  it("rejects workflow command defaults", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "concurrency:\n",
          "defaults:\n  run:\n    shell: bash\nconcurrency:\n"
        )
      )
    ).toThrow("workflow.defaults.run must not alter parity command execution");
  });

  it("rejects an added OpenSpec preparation step", () => {
    expect(() =>
      validateCiGates(
        insertStepBefore(
          workflow,
          "Validate living specs and active changes",
          "      - name: Prepare policy bypass\n        run: echo OPENCUT_UPDATE_GOLDENS=1 >> \"$GITHUB_ENV\"\n"
        )
      )
    ).toThrow("openspec must contain exactly its approved step sequence");
  });

  it("rejects reordered OpenSpec steps", () => {
    expect(() => validateCiGates(moveOpenSpecValidationBeforeToolchain(workflow))).toThrow(
      "openspec.steps[1] must be"
    );
  });

  for (const [description, needle, replacement, expected] of [
    [
      "changed bootstrap Bun action",
      "uses: oven-sh/setup-bun@v2",
      "uses: oven-sh/setup-bun@v1",
      "openspec bootstrap Bun step must use the unconditional pinned setup action",
    ],
    [
      "changed bootstrap Bun version",
      "bun-version: 1.4.0",
      "bun-version: 1.4.1",
      'openspec bootstrap Bun version must equal "1.4.0"',
    ],
    [
      "enabled Moon auto-install",
      "auto-install: false",
      "auto-install: true",
      "openspec bootstrap Moon must use the exact isolated setup options",
    ],
    [
      "enabled Moon auto-setup",
      "auto-setup: false",
      "auto-setup: true",
      "openspec bootstrap Moon must use the exact isolated setup options",
    ],
    [
      "enabled Moon bootstrap cache",
      "cache: false",
      "cache: true",
      "openspec bootstrap Moon must use the exact isolated setup options",
    ],
    [
      "changed bootstrap Moon version",
      "moon-version: 2.3.3",
      "moon-version: 2.3.4",
      "openspec bootstrap Moon must use the exact isolated setup options",
    ],
  ] as const) {
    it(`rejects ${description}`, () => {
      expect(() => validateCiGates(replaceRequired(workflow, needle, replacement))).toThrow(
        expected
      );
    });
  }

  it("rejects extra bootstrap setup options", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "          bun-version: 1.4.0\n",
          "          bun-version: 1.4.0\n          cache: true\n"
        )
      )
    ).toThrow("openspec bootstrap Bun with must contain exactly the approved properties");
  });

  it("rejects an altered OpenSpec validation command", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          `run: ${workflowBootstrapCommand}`,
          `run: ${workflowBootstrapCommand} || true`
        )
      )
    ).toThrow("openspec validation step must use the exact fail-closed command body");
  });

  it("rejects a missing OpenSpec policy step identifier", () => {
    expect(() =>
      validateCiGates(replaceRequired(workflow, "        id: policy\n", ""))
    ).toThrow("openspec validation step must contain exactly the approved properties");
  });

  it("rejects an altered OpenSpec policy step identifier", () => {
    expect(() =>
      validateCiGates(replaceRequired(workflow, "        id: policy\n", "        id: bypass\n"))
    ).toThrow('openspec validation step id must equal "policy"');
  });

  it("rejects a missing OpenSpec policy output", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "    outputs:\n      policy_validated: ${{ steps.policy.outputs.validated }}\n",
          ""
        )
      )
    ).toThrow("jobs.openspec must contain exactly the approved properties");
  });

  it("rejects an extra OpenSpec policy output", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "      policy_validated: ${{ steps.policy.outputs.validated }}\n",
          "      policy_validated: ${{ steps.policy.outputs.validated }}\n      forged: true\n"
        )
      )
    ).toThrow("jobs.openspec.outputs must contain exactly the approved properties");
  });

  for (const output of ["true", "${{ steps.other.outputs.validated }}"]) {
    it(`rejects forged OpenSpec policy output ${output}`, () => {
      expect(() =>
        validateCiGates(
          replaceRequired(
            workflow,
            "policy_validated: ${{ steps.policy.outputs.validated }}",
            `policy_validated: ${output}`
          )
        )
      ).toThrow("jobs.openspec policy_validated must expose the policy step attestation");
    });
  }

  for (const [description, command] of [
    ["a direct Moon target", "moon run root:openspec-validate"],
    [
      "inline output write",
      `${workflowBootstrapCommand}\n          echo \"validated=true\" >> \"$GITHUB_OUTPUT\"`,
    ],
    ["an additional command", `${workflowBootstrapCommand}\n          true`],
    ["a neutralized command", `${workflowBootstrapCommand} || true`],
    [
      "repository Bun config",
      "bun --config=bunfig.toml --no-env-file run scripts/run-ci-policy.ts --attest-github-output",
    ],
    [
      "automatic dotenv loading",
      "bun --config=/dev/null run scripts/run-ci-policy.ts --attest-github-output",
    ],
    [
      "default Bun config",
      "bun --no-env-file run scripts/run-ci-policy.ts --attest-github-output",
    ],
  ] as const) {
    it(`rejects OpenSpec workflow command with ${description}`, () => {
      expect(() =>
        validateCiGates(
          replaceRequired(
            workflow,
            `run: ${workflowBootstrapCommand}`,
            `run: |\n          ${command}`
          )
        )
      ).toThrow("openspec validation step must use the exact fail-closed command body");
    });
  }

  for (const property of ["shell: true {0}", "if: false", "working-directory: ."]) {
    it(`rejects OpenSpec validation property ${property}`, () => {
      expect(() =>
        validateCiGates(
          addStepProperty(workflow, "Validate living specs and active changes", property)
        )
      ).toThrow();
    });
  }

  it("rejects altered OpenSpec checkout history", () => {
    expect(() =>
      validateCiGates(replaceRequired(workflow, "fetch-depth: 0", "fetch-depth: 1"))
    ).toThrow("openspec checkout fetch-depth must equal 0");
  });

  it("rejects a conditional OpenSpec job", () => {
    expect(() =>
      validateCiGates(addJobConfiguration(workflow, "openspec", "    if: false\n"))
    ).toThrow("jobs.openspec must not be conditionally skipped");
  });

  it("rejects OpenSpec command defaults", () => {
    expect(() =>
      validateCiGates(
        addJobConfiguration(
          workflow,
          "openspec",
          "    defaults:\n      run:\n        shell: true {0}\n"
        )
      )
    ).toThrow("jobs.openspec.defaults.run must not alter parity command execution");
  });

  it("rejects an OpenSpec container", () => {
    expect(() =>
      validateCiGates(
        addJobConfiguration(
          workflow,
          "openspec",
          "    container:\n      image: ubuntu:latest\n"
        )
      )
    ).toThrow("jobs.openspec.container must not alter the reviewed runner environment");
  });

  for (const shell of ["bash", "true {0}"]) {
    it(`rejects aggregate custom shell ${shell}`, () => {
      expect(() =>
        validateCiGates(
          addStepProperty(workflow, "Confirm foundation parity gates", `shell: ${shell}`)
        )
      ).toThrow("foundation-parity assertion step must contain exactly the approved properties");
    });
  }

  it("rejects aggregate command defaults", () => {
    expect(() =>
      validateCiGates(
        addJobConfiguration(
          workflow,
          "foundation-parity",
          "    defaults:\n      run:\n        shell: true {0}\n"
        )
      )
    ).toThrow("jobs.foundation-parity.defaults.run must not alter parity command execution");
  });

  it("rejects an aggregate container", () => {
    expect(() =>
      validateCiGates(
        addJobConfiguration(
          workflow,
          "foundation-parity",
          "    container:\n      image: ubuntu:latest\n"
        )
      )
    ).toThrow("jobs.foundation-parity.container must not alter the reviewed runner environment");
  });

  it("rejects an unexpected aggregate assertion property", () => {
    expect(() =>
      validateCiGates(
        addStepProperty(workflow, "Confirm foundation parity gates", "timeout-minutes: 5")
      )
    ).toThrow("foundation-parity assertion step must contain exactly the approved properties");
  });

  it("rejects an additional aggregate result environment key", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "          CONTRACT_PARITY_RESULT: ${{ needs.contract-parity.result }}\n",
          "          CONTRACT_PARITY_RESULT: ${{ needs.contract-parity.result }}\n          EXTRA_RESULT: success\n"
        )
      )
    ).toThrow(
      "foundation-parity assertion env must contain exactly the approved environment keys"
    );
  });

  it("rejects a missing aggregate result environment key", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "          RENDER_PARITY_RESULT: ${{ needs.render-parity.result }}\n",
          ""
        )
      )
    ).toThrow(
      "foundation-parity assertion env must contain exactly the approved environment keys"
    );
  });

  it("rejects a missing aggregate policy attestation binding", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "          OPENSPEC_POLICY_VALIDATED: ${{ needs.openspec.outputs.policy_validated }}\n",
          ""
        )
      )
    ).toThrow(
      "foundation-parity assertion env must contain exactly the approved environment keys"
    );
  });

  it("rejects a forged aggregate policy attestation binding", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "OPENSPEC_POLICY_VALIDATED: ${{ needs.openspec.outputs.policy_validated }}",
          "OPENSPEC_POLICY_VALIDATED: true"
        )
      )
    ).toThrow("foundation-parity must expose the openspec policy attestation");
  });

  it("rejects an unexpected property on an approved leaf step", () => {
    expect(() =>
      validateCiGates(
        addStepProperty(
          workflow,
          "Install deterministic rendering dependencies",
          "timeout-minutes: 5"
        )
      )
    ).toThrow("must contain exactly the approved properties");
  });

  for (const variable of [
    "OPENCUT_UPDATE_GOLDENS",
    "OPENCUT_CAPTURE_GOLDENS_TO",
  ]) {
    for (const scope of [
      "workflow",
      "job",
      "native",
      "validation",
    ] as const) {
      it(`rejects ${variable} at ${scope} scope regardless of value`, () => {
        for (const value of ["1", "0", "disabled"]) {
          expect(() =>
            validateCiGates(addEnvironmentVariable(workflow, scope, variable, value))
          ).toThrow(`must not declare verification-bypassing ${variable}`);
        }
      });
    }
  }

  it("rejects an additional native conformance environment key", () => {
    expect(() =>
      validateCiGates(
        addEnvironmentVariable(workflow, "native", "OPENCUT_UNEXPECTED", "value")
      )
    ).toThrow("render-parity native env must contain exactly the approved environment keys");
  });

  it("rejects an additional report-validation environment key", () => {
    expect(() =>
      validateCiGates(
        addEnvironmentVariable(workflow, "validation", "OPENCUT_UNEXPECTED", "value")
      )
    ).toThrow(
      "render-parity report-validation env must contain exactly the approved environment keys"
    );
  });

  it("rejects a missing leaf job", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "  contract-parity:\n",
          "  renamed-contract-parity:\n"
        )
      )
    ).toThrow("workflow must define jobs.contract-parity");
  });

  it("rejects a missing aggregate dependency", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "needs: [openspec, contract-parity, render-parity]",
          "needs: [contract-parity, render-parity]"
        )
      )
    ).toThrow("must contain exactly openspec, contract-parity, and render-parity");
  });

  it("rejects an altered unconditional aggregate condition", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(workflow, "if: ${{ always() }}", "if: ${{ !cancelled() }}")
      )
    ).toThrow("jobs.foundation-parity.if must equal");
  });

  for (const [variable, jobId] of [
    ["OPENSPEC_RESULT", "openspec"],
    ["CONTRACT_PARITY_RESULT", "contract-parity"],
    ["RENDER_PARITY_RESULT", "render-parity"],
  ]) {
    it(`rejects a missing ${jobId} aggregate result`, () => {
      expect(() =>
        validateCiGates(
          replaceRequired(
            workflow,
            `${variable}: \${{ needs.${jobId}.result }}`,
            `${variable}: success`
          )
        )
      ).toThrow(`must expose the ${jobId} result`);
    });
  }

  it("rejects a weakened authoritative contract command", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "run: bun run contracts:check",
          "run: bun run contracts:check || true"
        )
      )
    ).toThrow("contract-parity command step must use the exact fail-closed command body");
  });

  it("rejects a neutralized native render command", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact",
          "cargo test -p opencut-editor-core renderer::golden::native_golden_render_conformance -- --exact || true"
        )
      )
    ).toThrow("render-parity native step must use the exact fail-closed command body");
  });

  for (const stepName of [
    "Validate living specs and active changes",
    "Cross-language contract parity",
    "Native audiovisual and lifecycle parity",
    "Validate Linux render baseline schema",
    "Upload report-only Linux render baseline",
    "Confirm foundation parity gates",
  ]) {
    it(`rejects continue-on-error on ${stepName}`, () => {
      expect(() => validateCiGates(addContinueOnError(workflow, stepName))).toThrow(
        "must not ignore failures with continue-on-error"
      );
    });
  }

  for (const jobId of [
    "openspec",
    "contract-parity",
    "render-parity",
    "foundation-parity",
  ]) {
    it(`rejects job-level continue-on-error on ${jobId}`, () => {
      expect(() =>
        validateCiGates(
          replaceRequired(
            workflow,
            `  ${jobId}:\n`,
            `  ${jobId}:\n    continue-on-error: true\n`
          )
        )
      ).toThrow("must not ignore failures with continue-on-error");
    });
  }

  it("rejects the contract command outside its declared workspace", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "working-directory: apps/agent-bridge\n        run: bun run contracts:check",
          "working-directory: .\n        run: bun run contracts:check"
        )
      )
    ).toThrow('working-directory must equal "apps/agent-bridge"');
  });

  it("rejects a mismatched report upload", () => {
    expect(() =>
      validateCiGates(
        replaceRequired(
          workflow,
          "path: target/render-baseline-linux.json",
          "path: target/another-report.json"
        )
      )
    ).toThrow('upload path must equal "target/render-baseline-linux.json"');
  });

  it("rejects report upload before strict validation", () => {
    expect(() => validateCiGates(moveUploadBeforeValidation(workflow))).toThrow(
      "render-parity.steps[4] must be"
    );
  });

  it("propagates a policy-detected environment injection to the aggregate", () => {
    const injectedWorkflow = insertStepBefore(
      workflow,
      "Native audiovisual and lifecycle parity",
      "      - name: Persist golden update mode\n        run: echo OPENCUT_UPDATE_GOLDENS=1 >> \"$GITHUB_ENV\"\n"
    );
    expect(() => validateCiGates(injectedWorkflow)).toThrow(
      "render-parity must contain exactly its approved step sequence"
    );
    expect(() =>
      assertFoundationParityResults("failure", "success", "success", "")
    ).toThrow("foundation parity requires success results");
  });

  it("keeps the aggregate failed when ignored policy failure reports a successful job", () => {
    const ignoredFailureWorkflow = addContinueOnError(
      workflow,
      "Validate living specs and active changes"
    );
    expect(() => validateCiGates(ignoredFailureWorkflow)).toThrow(
      "must not ignore failures with continue-on-error"
    );
    expect(() =>
      assertFoundationParityResults("success", "success", "success", "")
    ).toThrow("policy_validated=");
  });
});

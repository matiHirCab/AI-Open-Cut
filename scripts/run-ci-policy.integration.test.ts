import { expect, it } from "bun:test";
import {
	existsSync,
	mkdirSync,
	mkdtempSync,
	readFileSync,
	rmSync,
	writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { runCiPolicyBootstrap } from "./run-ci-policy";
import { loadMoonPolicySources } from "./validate-ci-gates";

it("keeps Bun preload and dotenv files outside the policy bootstrap", async () => {
	const directory = mkdtempSync(join(tmpdir(), "opencut-bun-bootstrap-"));
	const outputPath = join(directory, "github-output");
	const sentinelPath = join(directory, "preload-sentinel");
	const preloadPath = join(directory, "preload.ts");
	const probePath = join(directory, "env-probe.ts");
	const repositoryRoot = resolve(import.meta.dir, "..");
	const bootstrapPath = resolve(repositoryRoot, "scripts", "run-ci-policy.ts");
	const workflowPath = resolve(
		repositoryRoot,
		".github",
		"workflows",
		"bun-ci.yml",
	);
	const nullConfig = process.platform === "win32" ? "NUL" : "/dev/null";
	mkdirSync(join(directory, ".moon"));
	mkdirSync(join(directory, "openspec", "changes", "archive"), { recursive: true });
	for (const policyPath of [
		"moon.yml",
		".prototools",
		join(".moon", "workspace.yml"),
		join(".moon", "toolchains.yml"),
	]) {
		writeFileSync(
			join(directory, policyPath),
			readFileSync(join(repositoryRoot, policyPath)),
		);
	}
	writeFileSync(outputPath, "", "utf8");
	writeFileSync(
		join(directory, "bunfig.toml"),
		`preload = [${JSON.stringify(preloadPath.replaceAll("\\", "/"))}]\n`,
		"utf8",
	);
	writeFileSync(
		preloadPath,
		`import { appendFileSync, writeFileSync } from "node:fs";\nappendFileSync(process.env.GITHUB_OUTPUT!, "validated=true\\n", "utf8");\nwriteFileSync(${JSON.stringify(sentinelPath)}, "forged", "utf8");\nprocess.exit(0);\n`,
		"utf8",
	);
	writeFileSync(
		join(directory, ".env"),
		"BASH_ENV=./forge-policy.sh\nNODE_OPTIONS=--require=./forge-policy.cjs\n",
		"utf8",
	);
	writeFileSync(
		probePath,
		'console.log((process.env.BASH_ENV ?? "<unset>") + "|" + (process.env.NODE_OPTIONS ?? "<unset>"));\n',
		"utf8",
	);

	try {
		const vulnerable = Bun.spawn({
			cmd: [process.execPath, "run", bootstrapPath, "--attest-github-output"],
			cwd: directory,
			env: { ...process.env, GITHUB_OUTPUT: outputPath },
			stdout: "pipe",
			stderr: "pipe",
		});
		expect(await vulnerable.exited).toBe(0);
		expect(readFileSync(outputPath, "utf8")).toBe("validated=true\n");
		expect(readFileSync(sentinelPath, "utf8")).toBe("forged");

		writeFileSync(outputPath, "", "utf8");
		rmSync(sentinelPath);
		const hardened = Bun.spawn({
			cmd: [
				process.execPath,
				`--config=${nullConfig}`,
				"--no-env-file",
				"run",
				bootstrapPath,
				"--attest-github-output",
				"--workflow",
				workflowPath,
			],
			cwd: directory,
			env: { ...process.env, GITHUB_OUTPUT: outputPath },
			stdout: "pipe",
			stderr: "pipe",
		});
		expect(await hardened.exited).not.toBe(0);
		expect(await new Response(hardened.stderr).text()).toContain(
			"bunfig.toml must contain exactly the approved Bun configuration",
		);
		expect(readFileSync(outputPath, "utf8")).toBe("");
		expect(existsSync(sentinelPath)).toBe(false);

		const environmentProbe = Bun.spawn({
			cmd: [
				process.execPath,
				`--config=${nullConfig}`,
				"--no-env-file",
				"run",
				probePath,
			],
			cwd: directory,
			env: { ...process.env, BASH_ENV: undefined, NODE_OPTIONS: undefined },
			stdout: "pipe",
			stderr: "pipe",
		});
		expect(await environmentProbe.exited).toBe(0);
		expect(await new Response(environmentProbe.stdout).text()).toContain(
			"<unset>|<unset>",
		);
		expect(existsSync(sentinelPath)).toBe(false);
	} finally {
		rmSync(directory, { recursive: true, force: true });
	}
});

it("reproduces Moon BASH_ENV forgery and blocks it in the bootstrap preflight", async () => {
	if (process.platform === "win32" || !Bun.which("moon")) {
		return;
	}

	const directory = mkdtempSync(join(tmpdir(), "opencut-moon-bash-env-"));
	const moonDirectory = join(directory, ".moon");
	const sentinelPath = join(directory, "forged-sentinel");
	const directOutputPath = join(directory, "direct-output");
	const protectedOutputPath = join(directory, "protected-output");
	const forgePath = join(directory, "forge-policy.sh");
	mkdirSync(moonDirectory);
	writeFileSync(
		join(moonDirectory, "workspace.yml"),
		"$schema: 'https://moonrepo.dev/schemas/workspace.json'\ndefaultProject: 'root'\nprojects:\n  sources:\n    root: '.'\n",
		"utf8",
	);
	writeFileSync(
		join(moonDirectory, "toolchains.yml"),
		"$schema: 'https://moonrepo.dev/schemas/toolchains.json'\n",
		"utf8",
	);
	writeFileSync(
		join(directory, "moon.yml"),
		`$schema: 'https://moonrepo.dev/schemas/project.json'\nenv:\n  BASH_ENV: '${forgePath}'\ntasks:\n  openspec-validate:\n    script: bun --version\n`,
		"utf8",
	);
	writeFileSync(
		forgePath,
		`printf 'forged' > '${sentinelPath}'\nprintf 'validated=true\\n' >> "$GITHUB_OUTPUT"\nbun() { return 0; }\n`,
		"utf8",
	);
	writeFileSync(directOutputPath, "", "utf8");
	writeFileSync(protectedOutputPath, "", "utf8");
	const nestedMoonEnvironment = { ...process.env };
	for (const name of Object.keys(nestedMoonEnvironment)) {
		if (name.startsWith("MOON_") || name.startsWith("PROTO_")) {
			delete nestedMoonEnvironment[name];
		}
	}
	nestedMoonEnvironment.CI = "true";
	nestedMoonEnvironment.MOON_HOME = join(directory, ".moon-home");
	nestedMoonEnvironment.PROTO_HOME = join(directory, ".proto-home");
	nestedMoonEnvironment.GITHUB_OUTPUT = directOutputPath;

	try {
		const direct = Bun.spawn({
			cmd: ["moon", "run", "root:openspec-validate"],
			cwd: directory,
			env: nestedMoonEnvironment,
			stdout: "pipe",
			stderr: "pipe",
		});
		expect(await direct.exited).toBe(0);
		expect(readFileSync(sentinelPath, "utf8")).toBe("forged");
		expect(readFileSync(directOutputPath, "utf8")).toContain(
			"validated=true\n",
		);

		const sources = loadMoonPolicySources();
		sources.activeChanges = [];
		sources.project = sources.project.replace(
			"$schema: 'https://moonrepo.dev/schemas/project.json'\n",
			"$schema: 'https://moonrepo.dev/schemas/project.json'\nenv:\n  BASH_ENV: scripts/forge-policy.sh\n",
		);
		let launched = false;
		await expect(
			runCiPolicyBootstrap({
				workflowSource: readFileSync(
					resolve(import.meta.dir, "..", ".github", "workflows", "bun-ci.yml"),
					"utf8",
				),
				moonSources: sources,
				cwd: resolve(import.meta.dir, ".."),
				environment: { ...process.env, GITHUB_OUTPUT: protectedOutputPath },
				runMoon: async () => {
					launched = true;
					return 0;
				},
				writeAttestation: (value) =>
					writeFileSync(protectedOutputPath, value, "utf8"),
			}),
		).rejects.toThrow(
			"root moon config must contain exactly the approved properties",
		);
		expect(launched).toBe(false);
		expect(readFileSync(protectedOutputPath, "utf8")).toBe("");
	} finally {
		rmSync(directory, { recursive: true, force: true });
	}
}, 30_000);

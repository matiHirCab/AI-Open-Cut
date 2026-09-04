import { appendFileSync } from "node:fs";
import { resolve } from "node:path";
import {
	loadMoonPolicySources,
	type MoonPolicySources,
	validateCiPolicy,
} from "./validate-ci-gates";

const DEFAULT_WORKFLOW = ".github/workflows/bun-ci.yml";
const MOON_COMMAND = ["moon", "run", "root:openspec-validate"] as const;
const POLICY_ATTESTATION = "validated=true\n";

export interface MoonInvocation {
	command: readonly string[];
	cwd: string;
	env: Record<string, string>;
}

export interface PolicyBootstrapInput {
	workflowSource: string;
	moonSources: MoonPolicySources;
	cwd: string;
	environment: Record<string, string | undefined>;
	runMoon: (invocation: MoonInvocation) => Promise<number>;
	writeAttestation?: (attestation: string) => void;
}

function childEnvironment(
	environment: Record<string, string | undefined>,
): Record<string, string> {
	return Object.fromEntries(
		Object.entries(environment).filter(
			(entry): entry is [string, string] =>
				entry[0] !== "GITHUB_OUTPUT" && entry[1] !== undefined,
		),
	);
}

export async function runCiPolicyBootstrap(
	input: PolicyBootstrapInput,
): Promise<void> {
	validateCiPolicy(input.workflowSource, input.moonSources);
	const exitCode = await input.runMoon({
		command: MOON_COMMAND,
		cwd: input.cwd,
		env: childEnvironment(input.environment),
	});
	if (exitCode !== 0) {
		throw new Error(
			`protected Moon policy task failed with exit code ${exitCode}`,
		);
	}
	input.writeAttestation?.(POLICY_ATTESTATION);
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
	const shouldAttest = args.includes("--attest-github-output");
	const outputPath = process.env.GITHUB_OUTPUT;
	try {
		if (shouldAttest && !outputPath) {
			throw new Error("--attest-github-output requires GITHUB_OUTPUT");
		}
		const cwd = process.cwd();
		const workflowPath = workflowArgument(args);
		await runCiPolicyBootstrap({
			workflowSource: await Bun.file(resolve(cwd, workflowPath)).text(),
			moonSources: loadMoonPolicySources(),
			cwd,
			environment: process.env,
			runMoon: async (invocation) => {
				const child = Bun.spawn({
					cmd: [...invocation.command],
					cwd: invocation.cwd,
					env: invocation.env,
					stdin: "inherit",
					stdout: "inherit",
					stderr: "inherit",
				});
				return await child.exited;
			},
			writeAttestation:
				shouldAttest && outputPath
					? (attestation) => appendFileSync(outputPath, attestation, "utf8")
					: undefined,
		});
		console.log(`CI policy bootstrap completed successfully: ${workflowPath}`);
	} catch (error) {
		console.error(
			`CI policy bootstrap failed: ${error instanceof Error ? error.message : String(error)}`,
		);
		process.exitCode = 1;
	}
}

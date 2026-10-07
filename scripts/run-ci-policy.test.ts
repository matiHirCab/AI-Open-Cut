import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { type MoonInvocation, runCiPolicyBootstrap } from "./run-ci-policy";
import { loadMoonPolicySources } from "./validate-ci-gates";

const root = resolve(import.meta.dir, "..");
const workflow = readFileSync(
	resolve(root, ".github", "workflows", "bun-ci.yml"),
	"utf8",
);

function archivedOnlySources() {
	const sources = loadMoonPolicySources();
	sources.activeChanges = [];
	return sources;
}

describe("CI policy bootstrap", () => {
	it("validates before launching the exact Moon target and attests after success", async () => {
		const events: string[] = [];
		let invocation: MoonInvocation | undefined;
		await runCiPolicyBootstrap({
			workflowSource: workflow,
			moonSources: archivedOnlySources(),
			cwd: root,
			environment: { PATH: "bootstrap-path", GITHUB_OUTPUT: "private-output" },
			runMoon: async (value) => {
				events.push("moon");
				invocation = value;
				return 0;
			},
			writeAttestation: (value) => events.push(value),
		});

		expect(invocation?.command).toEqual([
			"moon",
			"run",
			"root:openspec-validate",
		]);
		expect(invocation?.cwd).toBe(root);
		expect(invocation?.env.PATH).toBe("bootstrap-path");
		expect(invocation?.env.GITHUB_OUTPUT).toBeUndefined();
		expect(events).toEqual(["moon", "validated=true\n"]);
	});

	it("does not launch Moon or attest when preflight validation fails", async () => {
		let launched = false;
		let output = "";
		const sources = archivedOnlySources();
		sources.project = sources.project.replace(
			"$schema: 'https://moonrepo.dev/schemas/project.json'\n",
			"$schema: 'https://moonrepo.dev/schemas/project.json'\nenv:\n  BASH_ENV: scripts/forge-policy.sh\n",
		);

		await expect(
			runCiPolicyBootstrap({
				workflowSource: workflow,
				moonSources: sources,
				cwd: root,
				environment: { GITHUB_OUTPUT: "private-output" },
				runMoon: async () => {
					launched = true;
					return 0;
				},
				writeAttestation: (value) => {
					output += value;
				},
			}),
		).rejects.toThrow(
			"root moon config must contain exactly the approved properties",
		);
		expect(launched).toBe(false);
		expect(output).toBe("");
	});

	it("does not launch Moon or attest while an OpenSpec change is unarchived", async () => {
		let launched = false;
		let output = "";
		const sources = archivedOnlySources();
		sources.activeChanges = ["pending-change", "notes.txt"];

		await expect(
			runCiPolicyBootstrap({
				workflowSource: workflow,
				moonSources: sources,
				cwd: root,
				environment: { GITHUB_OUTPUT: "private-output" },
				runMoon: async () => {
					launched = true;
					return 0;
				},
				writeAttestation: (value) => {
					output += value;
				},
			}),
		).rejects.toThrow(
			"unarchived OpenSpec changes block merge readiness: notes.txt, pending-change",
		);
		expect(launched).toBe(false);
		expect(output).toBe("");
	});

	it("does not attest a nonzero Moon exit", async () => {
		let output = "";
		await expect(
			runCiPolicyBootstrap({
				workflowSource: workflow,
				moonSources: archivedOnlySources(),
				cwd: root,
				environment: { GITHUB_OUTPUT: "private-output" },
				runMoon: async () => 17,
				writeAttestation: (value) => {
					output += value;
				},
			}),
		).rejects.toThrow("protected Moon policy task failed with exit code 17");
		expect(output).toBe("");
	});

	it("does not attest when the Moon process cannot start", async () => {
		let output = "";
		await expect(
			runCiPolicyBootstrap({
				workflowSource: workflow,
				moonSources: archivedOnlySources(),
				cwd: root,
				environment: { GITHUB_OUTPUT: "private-output" },
				runMoon: async () => {
					throw new Error("spawn failed");
				},
				writeAttestation: (value) => {
					output += value;
				},
			}),
		).rejects.toThrow("spawn failed");
		expect(output).toBe("");
	});

	it("supports local execution without an output writer", async () => {
		await expect(
			runCiPolicyBootstrap({
				workflowSource: workflow,
				moonSources: archivedOnlySources(),
				cwd: root,
				environment: {},
				runMoon: async () => 0,
			}),
		).resolves.toBeUndefined();
	});
});

describe("required complete desktop contracts command authority", () => {
	for (const [label, command] of [
		["omitted desktop", "desktop"],
		["substituted desktop", "substitute"],
		["masked desktop", "mask"],
		["weakened predecessor", "predecessor"],
		["missing command", "missing"],
		["invalid package", "malformed"],
	] as const) {
		it(`rejects ${label} before Moon and attestation`, async () => {
			const sources=archivedOnlySources();
			const pkg=JSON.parse(sources.bridgePackage);
			const previous=pkg.scripts["contracts:check"] as string;
			const next=command==="desktop" ? previous.replace(" && cargo test -p opencut-desktop desktop_compositing","")
				: command==="substitute" ? previous.replace("desktop_compositing","startup_and_load")
				: command==="mask" ? previous.replace("cargo test -p opencut-desktop desktop_compositing","cargo test -p opencut-desktop desktop_compositing || true")
				: command==="predecessor" ? previous.replace(" --test mask_models","") : undefined;
			pkg.scripts["contracts:check"]=next;
			sources.bridgePackage=command==="malformed" ? "{" : JSON.stringify(pkg);
			let launched=false,attested=false;
			await expect(runCiPolicyBootstrap({workflowSource:workflow,moonSources:sources,cwd:root,environment:{},runMoon:async()=>{launched=true;return 0;},writeAttestation:()=>{attested=true;}})).rejects.toThrow(command==="malformed"?"valid JSON":"exact complete canonical command");
			expect(launched).toBe(false);expect(attested).toBe(false);
		});
	}
	it("rejects missing package tracked input before Moon and attestation",async()=>{
		const sources=archivedOnlySources();sources.project=sources.project.replace("      - 'apps/agent-bridge/package.json'\n","");let launched=false,attested=false;
		await expect(runCiPolicyBootstrap({workflowSource:workflow,moonSources:sources,cwd:root,environment:{},runMoon:async()=>{launched=true;return 0;},writeAttestation:()=>{attested=true;}})).rejects.toThrow();expect(launched).toBe(false);expect(attested).toBe(false);
	});
});

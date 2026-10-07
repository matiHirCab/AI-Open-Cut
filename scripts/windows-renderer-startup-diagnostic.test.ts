import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

test("focused Windows evidence executes the exact unchanged descendant assertion", () => {
	const source = readFileSync(
		fileURLToPath(
			new URL(
				"../.github/workflows/windows-renderer-startup-diagnostic.yml",
				import.meta.url,
			),
		),
		"utf8",
	);
	const workflow = Bun.YAML.parse(source) as {
		on: { pull_request: { branches: string[] } };
		permissions: { contents: string };
		jobs: Record<
			string,
			{
				"runs-on": string;
				"timeout-minutes": number;
				steps: Array<Record<string, unknown>>;
			}
		>;
	};
	expect(workflow.on.pull_request.branches).toEqual(["main"]);
	expect(workflow.permissions).toEqual({ contents: "read" });
	expect(Object.keys(workflow.jobs)).toEqual(["startup-evidence"]);
	const job = workflow.jobs["startup-evidence"];
	expect(job["runs-on"]).toBe("windows-latest");
	expect(job["timeout-minutes"]).toBe(15);
	expect(job.steps[1]).toEqual({
		name: "Setup pinned Bun",
		uses: "oven-sh/setup-bun@v2",
		with: { "bun-version": "1.4.0" },
	});
	expect(job.steps[2].run).toBe(
		"bun --config=bunfig.toml --no-env-file test scripts/windows-renderer-startup-diagnostic.test.ts",
	);
	expect(job.steps[3].run).toBe(
		"rustup toolchain install 1.97.0 --profile minimal",
	);
	expect(job.steps[4].run).toBe(
		"cargo +1.97.0 test -p opencut-headless --test render_worker crashed_worker_terminates_renderer_descendants -- --exact --nocapture",
	);
	expect(job.steps[4].env).toEqual({
		OPENCUT_WINDOWS_STARTUP_COMPARISON: "1",
		RUST_BACKTRACE: "1",
	});
	for (const required of ["bun-ci.yml", "rules-screen-full.yml"]) {
		expect(
			readFileSync(
				fileURLToPath(
					new URL(`../.github/workflows/${required}`, import.meta.url),
				),
				"utf8",
			),
		).not.toContain("OPENCUT_WINDOWS_STARTUP_COMPARISON");
	}
	const pins = readFileSync(
		fileURLToPath(new URL("../.prototools", import.meta.url)),
		"utf8",
	);
	expect(pins).toContain('bun  = "1.4.0"');
	expect(pins).toContain('rust = "1.97.0"');
	for (const step of job.steps) {
		expect(step["continue-on-error"]).toBeUndefined();
		expect(step.if).toBeUndefined();
	}
});

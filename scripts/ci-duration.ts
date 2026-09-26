export interface DurationException {
	owner: string;
	reason: string;
	baselineMinutes: number;
	evidenceUrl: string;
	expiresOn: string;
	capMinutes: number;
}

export interface DurationResult {
	elapsedMinutes: number;
	budgetMinutes: number;
	withinBudget: boolean;
	exception: DurationException | null;
}

const DEFAULT_BUDGET_MINUTES = 120;
const MAX_EXCEPTION_MINUTES = 135;

function utcTime(value: unknown, label: string): number {
	if (
		typeof value !== "string" ||
		!/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?Z$/.test(value)
	) {
		throw new Error(`${label} must be a UTC ISO timestamp`);
	}
	const parsed = Date.parse(value);
	if (
		!Number.isFinite(parsed) ||
		new Date(parsed).toISOString().slice(0, 19) !== value.slice(0, 19)
	) {
		throw new Error(`${label} is invalid`);
	}
	return parsed;
}

export function validateDurationException(
	value: DurationException,
	observedAt: string,
): void {
	const fields = Object.keys(value).sort();
	if (
		fields.join(",") !==
		[
			"baselineMinutes",
			"capMinutes",
			"evidenceUrl",
			"expiresOn",
			"owner",
			"reason",
		]
			.sort()
			.join(",")
	) {
		throw new Error("duration exception must have exactly the reviewed fields");
	}
	if (!value.owner.trim() || !value.reason.trim()) {
		throw new Error("duration exception must name an owner and reason");
	}
	if (
		!Number.isFinite(value.baselineMinutes) ||
		value.baselineMinutes <= DEFAULT_BUDGET_MINUTES ||
		!Number.isFinite(value.capMinutes) ||
		value.capMinutes <= DEFAULT_BUDGET_MINUTES ||
		value.capMinutes > MAX_EXCEPTION_MINUTES ||
		value.baselineMinutes > value.capMinutes
	) {
		throw new Error("duration exception baseline or cap is invalid");
	}
	if (
		!/^https:\/\/github\.com\/[^/]+\/[^/]+\/actions\/runs\/\d+$/.test(
			value.evidenceUrl,
		)
	) {
		throw new Error(
			"duration exception evidence must link to a GitHub Actions run",
		);
	}
	if (!/^\d{4}-\d\d-\d\d$/.test(value.expiresOn)) {
		throw new Error("duration exception expiry must be a UTC date");
	}
	const expiry = new Date(`${value.expiresOn}T00:00:00Z`);
	if (
		Number.isNaN(expiry.getTime()) ||
		expiry.toISOString().slice(0, 10) !== value.expiresOn
	) {
		throw new Error("duration exception expiry is invalid");
	}
	if (utcTime(observedAt, "observation") >= expiry.getTime() + 86_400_000) {
		throw new Error("duration exception has expired");
	}
}

export function evaluateDuration(
	runStartedAt: unknown,
	observedAt: string,
	exception: DurationException | null,
): DurationResult {
	const start = utcTime(runStartedAt, "run start");
	const end = utcTime(observedAt, "observation");
	if (end < start) {
		throw new Error("duration observation precedes run start");
	}
	if (exception) {
		validateDurationException(exception, observedAt);
	}
	const budgetMinutes = exception?.capMinutes ?? DEFAULT_BUDGET_MINUTES;
	const elapsedMinutes = (end - start) / 60_000;
	return {
		elapsedMinutes,
		budgetMinutes,
		withinBudget: elapsedMinutes <= budgetMinutes,
		exception,
	};
}

function requiredEnv(name: string): string {
	const value = process.env[name];
	if (!value) throw new Error(`${name} is required`);
	return value;
}

export async function fetchRunStartedAt(
	repository: string,
	runId: string,
	token: string,
	client: typeof fetch = fetch,
): Promise<unknown> {
	if (
		!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository) ||
		!/^\d+$/.test(runId)
	) {
		throw new Error("GitHub repository or run ID is invalid");
	}
	if (!token) throw new Error("GitHub token is required");
	const response = await client(
		`https://api.github.com/repos/${repository}/actions/runs/${runId}`,
		{
			headers: {
				Accept: "application/vnd.github+json",
				Authorization: `Bearer ${token}`,
				"X-GitHub-Api-Version": "2022-11-28",
			},
		},
	);
	if (!response.ok)
		throw new Error(
			`GitHub Actions run lookup failed: HTTP ${response.status}`,
		);
	const run = (await response.json()) as { run_started_at?: unknown };
	return run.run_started_at;
}

async function main(): Promise<void> {
	const runStartedAt = await fetchRunStartedAt(
		requiredEnv("GITHUB_REPOSITORY"),
		requiredEnv("GITHUB_RUN_ID"),
		requiredEnv("GH_TOKEN"),
	);
	const exception: DurationException = {
		owner: requiredEnv("CI_DURATION_OWNER"),
		reason: requiredEnv("CI_DURATION_REASON"),
		baselineMinutes: Number(requiredEnv("CI_DURATION_BASELINE_MINUTES")),
		evidenceUrl: requiredEnv("CI_DURATION_EVIDENCE_URL"),
		expiresOn: requiredEnv("CI_DURATION_EXPIRES_ON"),
		capMinutes: Number(requiredEnv("CI_DURATION_CAP_MINUTES")),
	};
	const result = evaluateDuration(
		runStartedAt,
		new Date().toISOString(),
		exception,
	);
	console.log(
		`Protected CI elapsed: ${result.elapsedMinutes.toFixed(2)} minutes`,
	);
	console.log(
		`Effective budget: ${result.budgetMinutes} minutes (default 120)`,
	);
	console.log(
		`Exception: ${exception.reason}; owner: ${exception.owner}; expires: ${exception.expiresOn}`,
	);
	console.log(
		`Baseline: ${exception.baselineMinutes} minutes; evidence: ${exception.evidenceUrl}`,
	);
	if (!result.withinBudget)
		throw new Error("protected CI duration exceeded its effective budget");
}

if (import.meta.main) {
	main().catch((error: unknown) => {
		console.error(error instanceof Error ? error.message : String(error));
		process.exitCode = 1;
	});
}

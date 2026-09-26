import { describe, expect, it } from "bun:test";
import {
	type DurationException,
	evaluateDuration,
	fetchRunStartedAt,
} from "./ci-duration";

const exception: DurationException = {
	owner: "@matiHirCab",
	reason: "1920x1080 rules-screen render baseline",
	baselineMinutes: 123.65,
	evidenceUrl:
		"https://github.com/matiHirCab/AI-Open-Cut/actions/runs/36244746913",
	expiresOn: "2026-10-26",
	capMinutes: 135,
};
const start = "2026-09-26T00:00:00Z";
const at = (minutes: number) =>
	new Date(Date.parse(start) + minutes * 60_000).toISOString();

describe("protected CI duration", () => {
	it("enforces the default budget at its exact boundary", () => {
		expect(evaluateDuration(start, at(120), null).withinBudget).toBe(true);
		expect(evaluateDuration(start, at(120.01), null).withinBudget).toBe(false);
	});

	it("enforces the justified hard cap at its exact boundary", () => {
		expect(evaluateDuration(start, at(135), exception).withinBudget).toBe(true);
		expect(evaluateDuration(start, at(135.01), exception).withinBudget).toBe(
			false,
		);
	});

	it("rejects missing, invalid, and future start times", () => {
		expect(() => evaluateDuration(undefined, at(1), null)).toThrow();
		expect(() => evaluateDuration("bad", at(1), null)).toThrow();
		expect(() =>
			evaluateDuration("2026-02-30T00:00:00Z", at(1), null),
		).toThrow();
		expect(() => evaluateDuration(at(2), at(1), null)).toThrow();
	});

	it("rejects malformed, oversized, and expired exceptions", () => {
		expect(() =>
			evaluateDuration(start, at(1), { ...exception, capMinutes: 136 }),
		).toThrow();
		expect(() =>
			evaluateDuration(start, at(1), { ...exception, reason: "" }),
		).toThrow();
		expect(() =>
			evaluateDuration(start, at(1), { ...exception, expiresOn: "2026-02-30" }),
		).toThrow();
		expect(() =>
			evaluateDuration(
				"2026-10-27T00:00:00Z",
				"2026-10-27T00:01:00Z",
				exception,
			),
		).toThrow("expired");
	});

	it("fails closed when the Actions API is unavailable or omits the run start", async () => {
		const unavailable = (() =>
			Promise.resolve(
				new Response("unavailable", { status: 503 }),
			)) as typeof fetch;
		await expect(
			fetchRunStartedAt("owner/repo", "123", "read-token", unavailable),
		).rejects.toThrow("HTTP 503");
		const missing = (() =>
			Promise.resolve(
				Response.json({ status: "in_progress" }),
			)) as typeof fetch;
		const timestamp = await fetchRunStartedAt(
			"owner/repo",
			"123",
			"read-token",
			missing,
		);
		expect(() => evaluateDuration(timestamp, at(1), null)).toThrow("run start");
	});

	it("reads the exact current Actions run with the read-only job token", async () => {
		let requestedUrl = "";
		let authorization = "";
		const client = ((input: RequestInfo | URL, init?: RequestInit) => {
			requestedUrl = String(input);
			authorization = new Headers(init?.headers).get("Authorization") ?? "";
			return Promise.resolve(Response.json({ run_started_at: start }));
		}) as typeof fetch;
		expect(
			await fetchRunStartedAt("owner/repo", "123", "read-token", client),
		).toBe(start);
		expect(requestedUrl).toBe(
			"https://api.github.com/repos/owner/repo/actions/runs/123",
		);
		expect(authorization).toBe("Bearer read-token");
	});
});

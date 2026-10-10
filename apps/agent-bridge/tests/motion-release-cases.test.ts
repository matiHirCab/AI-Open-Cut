import { describe, expect, it } from "vitest";
import {
  RELEASE_AREAS,
  RELEASE_CASES,
  releaseCaseArguments,
  requireExactRustCase,
  requireReleaseInventory,
} from "../scripts/motion-release-cases";

const inventory = () => ({
  cases: RELEASE_CASES.map((row) => ({ ...row })),
  version: 1,
});
const firstCase = (value: ReturnType<typeof inventory>) => {
  const [first] = value.cases;
  if (!first) {
    throw new Error("control case absent");
  }
  return first;
};
const success =
  "running 1 test\ntest canonical::boundary ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 702 filtered out; finished in 0.01s\n";
describe("mandatory canonical release inventory", () => {
  it("covers exactly eight areas and selects every case without feature substitution", () => {
    requireReleaseInventory(inventory());
    expect(new Set(RELEASE_CASES.map((row) => row.area))).toEqual(
      new Set(RELEASE_AREAS)
    );
    for (const row of RELEASE_CASES) {
      const args = releaseCaseArguments(row);
      expect(args).toContain("--release");
      expect(args).toContain("--no-default-features");
      expect(args).toContain("--exact");
      expect(args).not.toContain("--features");
      expect(args.includes("--ignored")).toBe(row.ignored);
    }
  });
  for (const area of RELEASE_AREAS) {
    it(`refuses missing ${area} evidence`, () => {
      const broken = inventory();
      broken.cases = broken.cases.filter((row) => row.area !== area);
      expect(() => requireReleaseInventory(broken)).toThrow();
    });
  }
  it("rejects duplicates, unknown fields and target/name/ignore substitutions", () => {
    for (const change of [
      (v: ReturnType<typeof inventory>) => {
        v.cases[1] = { ...firstCase(v) };
      },
      (v: ReturnType<typeof inventory>) => {
        firstCase(v).target = "unrelated";
      },
      (v: ReturnType<typeof inventory>) => {
        firstCase(v).selector = "";
      },
      (v: ReturnType<typeof inventory>) => {
        firstCase(v).ignored = true;
      },
    ]) {
      const broken = inventory();
      change(broken);
      expect(() => requireReleaseInventory(broken)).toThrow();
    }
    expect(() =>
      requireReleaseInventory({ ...inventory(), skip: true })
    ).toThrow();
    expect(() =>
      requireReleaseInventory({ ...inventory(), version: 2 })
    ).toThrow();
    expect(() => requireReleaseInventory(null)).toThrow();
    const extra = inventory();
    Object.assign(firstCase(extra), { optional: true });
    expect(() => requireReleaseInventory(extra)).toThrow();
  });
  it("requires the actual exact passing case on LF or Windows CRLF", () => {
    requireExactRustCase("canonical::boundary", success, 0);
    requireExactRustCase(
      "canonical::boundary",
      success.replaceAll("\n", "\r\n"),
      0
    );
  });
  it("rejects success with zero tests, ignores, unrelated names, failed status or duplicates", () => {
    for (const output of [
      "test result: ok. 0 passed; 0 failed; 0 ignored;",
      success
        .replace("... ok", "... ignored")
        .replace(
          "1 passed; 0 failed; 0 ignored;",
          "0 passed; 0 failed; 1 ignored;"
        ),
      success.replace("canonical::boundary", "unrelated::boundary"),
      success.replace("... ok", "... FAILED"),
      success + success,
      `${success}test result: FAILED. 0 passed; 1 failed; 0 ignored;`,
    ]) {
      expect(() =>
        requireExactRustCase("canonical::boundary", output, 0)
      ).toThrow();
    }
    expect(() =>
      requireExactRustCase("canonical::boundary", success, 1)
    ).toThrow();
  });
});

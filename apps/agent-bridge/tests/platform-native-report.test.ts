import { expect, it } from "vitest";
import { requirePlatformNativeReport } from "../scripts/platform-native-report";

const valid = () => ({
  numFailedTests: 0,
  numPassedTests: 2,
  numPendingTests: 0,
  numTotalTests: 2,
  success: true,
  testResults: [
    {
      assertionResults: [
        {
          fullName:
            "actual default source renderer availability, fallback and media",
          status: "passed",
        },
        {
          fullName:
            "actual default packaged renderer availability, fallback and media",
          status: "passed",
        },
      ],
    },
  ],
});
it("accepts exactly both observed native cases", () =>
  expect(() => requirePlatformNativeReport(valid())).not.toThrow());
it.each([
  null,
  {},
  { ...valid(), numPassedTests: 0, numPendingTests: 2 },
  { ...valid(), numFailedTests: 1, numPassedTests: 1 },
  { ...valid(), numTotalTests: 0 },
  { ...valid(), success: false },
  { ...valid(), testResults: [] },
])("rejects incomplete/failed native report %j", (report) =>
  expect(() => requirePlatformNativeReport(report)).toThrow()
);
it("rejects renamed or skipped assertions even if summary counts claim success", () => {
  const report = valid();
  const assertion = report.testResults
    .flatMap((suite) => suite.assertionResults)
    .find((entry) => entry.fullName.includes("source"));
  if (!assertion) {
    throw new Error("missing report fixture");
  }
  assertion.status = "pending";
  expect(() => requirePlatformNativeReport(report)).toThrow();
  assertion.status = "passed";
  assertion.fullName = "unrelated native test";
  expect(() => requirePlatformNativeReport(report)).toThrow();
});

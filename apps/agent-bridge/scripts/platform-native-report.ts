const EXPECTED = [
  "actual default source renderer availability, fallback and media",
  "actual default packaged renderer availability, fallback and media",
];

/** Require both native cases to execute; a successful all-skipped run is failure. */
export const requirePlatformNativeReport = (value: unknown) => {
  if (typeof value !== "object" || value === null) {
    throw new Error("Missing platform native execution report");
  }
  const report = value as Record<string, unknown>;
  if (
    report.success !== true ||
    report.numTotalTests !== 2 ||
    report.numPassedTests !== 2 ||
    report.numFailedTests !== 0 ||
    report.numPendingTests !== 0 ||
    !Array.isArray(report.testResults) ||
    report.testResults.length !== 1
  ) {
    throw new Error("Both platform native cases must pass without skips");
  }
  const [suite] = report.testResults as {
    assertionResults?: { fullName?: string; status?: string }[];
  }[];
  const assertions = suite?.assertionResults;
  if (
    assertions?.length !== 2 ||
    !EXPECTED.every((name) =>
      assertions.some(
        (assertion) =>
          assertion.fullName === name && assertion.status === "passed"
      )
    )
  ) {
    throw new Error("Unexpected or skipped platform native cases");
  }
};

const NAMES = [
  "native ten-group reference through source MCP",
  "native ten-group reference through compiled MCP",
];
export const requireMotionMcpReport = (value: unknown) => {
  if (typeof value !== "object" || value === null) {
    throw new Error("Missing complete MCP execution report");
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
    throw new Error("Both actual complete MCP cases must pass without skips");
  }
  const [suite] = report.testResults as {
    assertionResults?: { fullName?: string; status?: string }[];
  }[];
  const assertions = suite?.assertionResults;
  if (
    assertions?.length !== 2 ||
    !NAMES.every(
      (name) =>
        assertions.filter(
          (row) => row.fullName === name && row.status === "passed"
        ).length === 1
    )
  ) {
    throw new Error("Missing, duplicate or skipped complete MCP cases");
  }
};

import { expect, it } from "vitest";
import { requireMotionMcpReport } from "../scripts/motion-mcp-report";

const healthy = () => ({
  numFailedTests: 0,
  numPassedTests: 2,
  numPendingTests: 0,
  numTotalTests: 2,
  success: true,
  testResults: [
    {
      assertionResults: [
        {
          fullName: "native ten-group reference through source MCP",
          status: "passed",
        },
        {
          fullName: "native ten-group reference through compiled MCP",
          status: "passed",
        },
      ],
    },
  ] as [{ assertionResults: { fullName: string; status: string }[] }],
});
it("requires both actual complete source and compiled native MCP cases", () => {
  expect(() => requireMotionMcpReport(healthy())).not.toThrow();
  for (const replacement of [
    { success: false },
    { numTotalTests: 0 },
    { numPassedTests: 0 },
    { numFailedTests: 1 },
    { numPendingTests: 2 },
    { testResults: [] },
  ]) {
    expect(() =>
      requireMotionMcpReport({ ...healthy(), ...replacement })
    ).toThrow();
  }
  for (const status of ["skipped", "pending", "failed"]) {
    const value = healthy();
    for (const suite of value.testResults) {
      for (const row of suite.assertionResults) {
        row.status = status;
      }
    }
    expect(() => requireMotionMcpReport(value)).toThrow();
  }
  const duplicate = healthy();
  const [suite] = duplicate.testResults;
  const first = {
    fullName: "native ten-group reference through source MCP",
    status: "passed",
  };
  suite.assertionResults = [first, first];
  expect(() => requireMotionMcpReport(duplicate)).toThrow();
  expect(() => requireMotionMcpReport(null)).toThrow();
});

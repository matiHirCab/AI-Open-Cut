/** Exact private counters for existing native identity support; no renderer override. */
const REUSE = [
  [0, 1, 1],
  [0, 2, 2],
  [0, 3, 3],
  [1, 3, 3],
  [1, 4, 4],
  [2, 4, 4],
];
const BYPASS = [
  [0, 0, 1],
  [0, 0, 2],
  [0, 0, 3],
  [0, 0, 4],
  [0, 0, 5],
  [0, 0, 6],
];
export const requireCompleteReferenceCacheTrace = (
  platform: string,
  value: unknown
) => {
  if (
    !(
      ["linux", "win32", "darwin"].includes(platform) && Array.isArray(value)
    ) ||
    value.length !== 6
  ) {
    throw new Error("Incomplete supported-platform native cache trace");
  }
  const expected = platform === "darwin" ? BYPASS : REUSE;
  const requests = new Set<string>();
  const actual = value.map((row: unknown) => {
    if (typeof row !== "object" || row === null || Array.isArray(row)) {
      throw new Error("Invalid native cache counters");
    }
    const counters = row as Record<string, unknown>;
    if (
      Object.keys(counters).sort().join(",") !==
      "finalExecutions,hits,misses,requestId"
    ) {
      throw new Error("Missing or unknown native cache counter");
    }
    if (
      typeof counters.requestId !== "string" ||
      counters.requestId.trim() === "" ||
      requests.has(counters.requestId)
    ) {
      throw new Error("Missing or duplicate native cache request identity");
    }
    requests.add(counters.requestId);
    return [counters.hits, counters.misses, counters.finalExecutions];
  });
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(
      "Native cache counters differ from exact platform semantics"
    );
  }
};

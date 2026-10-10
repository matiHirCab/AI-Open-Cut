import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { requireCompleteReferenceCacheTrace } from "../scripts/motion-native-cache-trace";

const recorded = (mode: string) => {
  const evidence = JSON.parse(
    readFileSync(
      resolve(
        import.meta.dirname,
        "../../../docs/verification/complete-reference-scene",
        `${mode}-native-evidence.json`
      ),
      "utf8"
    )
  ) as { records: { cacheStats?: unknown }[] };
  const value = evidence.records.find(
    (record) => record.cacheStats
  )?.cacheStats;
  if (!value) {
    throw new Error("Original native evidence lacks cache trace");
  }
  return value;
};
const macControl = () =>
  Array.from({ length: 6 }, (_, index) => ({
    finalExecutions: index + 1,
    hits: 0,
    misses: 0,
    requestId: `mac-validator-control-${index}`,
  }));
it("preserves original actual source and compiled ELF/PE traces", () => {
  for (const mode of ["source", "compiled"]) {
    for (const platform of ["linux", "win32"]) {
      expect(() =>
        requireCompleteReferenceCacheTrace(platform, recorded(mode))
      ).not.toThrow();
    }
  }
});
it("requires exact existing Mach-O bypass instead of pretending it reused the backend", () => {
  expect(() =>
    requireCompleteReferenceCacheTrace("darwin", macControl())
  ).not.toThrow();
  expect(() =>
    requireCompleteReferenceCacheTrace("darwin", recorded("source"))
  ).toThrow();
  expect(() =>
    requireCompleteReferenceCacheTrace("linux", macControl())
  ).toThrow();
  for (const index of [0, 1, 2, 3, 4, 5]) {
    for (const key of ["hits", "misses", "finalExecutions"]) {
      const rows = macControl();
      Object.assign(rows[index] ?? {}, { [key]: 77 });
      expect(() =>
        requireCompleteReferenceCacheTrace("darwin", rows)
      ).toThrow();
    }
  }
});
it("refuses missing, zero, duplicate, non-finite and substituted trace controls", () => {
  for (const value of [
    null,
    [],
    macControl().slice(1),
    [...macControl(), macControl()[0]],
    [...macControl().slice(0, 5), macControl()[0]],
    [{ hits: Number.NaN }, ...macControl().slice(1)],
  ]) {
    expect(() => requireCompleteReferenceCacheTrace("darwin", value)).toThrow();
  }
  for (const key of ["hits", "misses", "finalExecutions", "requestId"]) {
    const rows = macControl();
    const [first] = rows;
    if (!first) {
      throw new Error("control row absent");
    }
    delete (first as Record<string, unknown>)[key];
    expect(() => requireCompleteReferenceCacheTrace("darwin", rows)).toThrow();
  }
  const extra = macControl();
  Object.assign(extra[0] ?? {}, { hidden: true });
  expect(() => requireCompleteReferenceCacheTrace("darwin", extra)).toThrow();
  expect(() =>
    requireCompleteReferenceCacheTrace("unknown", macControl())
  ).toThrow();
  for (const requestId of ["", " ", null, 77, "mac-validator-control-1"]) {
    const rows = macControl();
    Object.assign(rows[0] ?? {}, { requestId });
    expect(() => requireCompleteReferenceCacheTrace("darwin", rows)).toThrow();
  }
});

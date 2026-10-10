import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import {
  assertDocumentationLinks,
  bindDocumentationArguments,
  documentationGuide,
  documentationRoot,
  parseDocumentationExample,
  readDocumentationExample,
} from "./release-documentation-fixture";

describe("release documentation corpus", () => {
  it("accepts the complete private corpus", () => {
    expect(readDocumentationExample().failures).toHaveLength(4);
  });
  it.each([
    "exampleVersion",
    "protocolVersion",
    "projectSchemaVersion",
  ] as const)("refuses incompatible %s", (field) => {
    const value: Record<string, unknown> = structuredClone(
      readDocumentationExample()
    );
    value[field] = Number(value[field]) + 1;
    expect(() => parseDocumentationExample(value)).toThrow();
  });
  it("refuses missing, duplicate and unknown refusal cases", () => {
    const value = structuredClone(readDocumentationExample());
    value.failures.pop();
    expect(() => parseDocumentationExample(value)).toThrow();
    const duplicate = structuredClone(readDocumentationExample());
    const [first] = duplicate.failures;
    if (!first) {
      throw new Error("missing fixture case");
    }
    duplicate.failures[1] = first;
    expect(() => parseDocumentationExample(duplicate)).toThrow();
    const unknown: Record<string, unknown>[] = structuredClone(
      readDocumentationExample().failures
    );
    const [unknownFirst] = unknown;
    if (!unknownFirst) {
      throw new Error("missing fixture case");
    }
    unknownFirst.id = "invented";
    expect(() =>
      parseDocumentationExample({
        ...readDocumentationExample(),
        failures: unknown,
      })
    ).toThrow();
  });
  it("refuses unknown metadata and capabilities", () => {
    expect(() =>
      parseDocumentationExample({ ...readDocumentationExample(), extra: true })
    ).toThrow();
    const value = structuredClone(readDocumentationExample());
    value.requiredEditorCapabilities[0] = "invented_capability";
    expect(() => parseDocumentationExample(value)).toThrow();
    const [, second] = value.requiredEditorCapabilities;
    if (!second) {
      throw new Error("missing capability");
    }
    value.requiredEditorCapabilities[0] = second;
    expect(() => parseDocumentationExample(value)).toThrow();
  });
  it("refuses retryability drift from the canonical error catalog", () => {
    const value = structuredClone(readDocumentationExample());
    const [first] = value.failures;
    if (!first) {
      throw new Error("missing fixture case");
    }
    first.expectedError.retryable = true;
    expect(() => parseDocumentationExample(value)).toThrow("retryability");
  });
  it("binds runtime IDs without changing aliases, literal values or the source", () => {
    const value = JSON.parse(
      '{"projectId":"$runtime:projectId","items":["@definition","ordinary"],"__proto__":{"safe":true}}'
    ) as Record<string, unknown>;
    const bound = bindDocumentationArguments(value, {
      projectId: "real-project",
    });
    expect(bound.projectId).toBe("real-project");
    expect(bound.items).toEqual(["@definition", "ordinary"]);
    expect(Object.hasOwn(bound, "__proto__")).toBe(true);
    expect(Object.getPrototypeOf(bound)).toBe(Object.prototype);
    expect(value.projectId).toBe("$runtime:projectId");
  });
  it("refuses unresolved bindings rather than guessing IDs", () => {
    expect(() =>
      bindDocumentationArguments({ id: "$runtime:missing" }, {})
    ).toThrow("unknown documentation runtime binding");
  });
  it("resolves every guide link and both entry points", () => {
    expect(assertDocumentationLinks(documentationGuide)).toBeGreaterThan(20);
    for (const name of ["README.md", "docs/agent-bridge.md"]) {
      expect(readFileSync(resolve(documentationRoot, name), "utf8")).toContain(
        "motion-graphics-release.md"
      );
    }
  });
  it("refuses missing or escaping local links", () => {
    expect(() =>
      assertDocumentationLinks(documentationGuide, "[bad](missing.md)")
    ).toThrow("missing or external");
    expect(() =>
      assertDocumentationLinks(documentationGuide, "[bad](../../../outside.md)")
    ).toThrow("missing or external");
    expect(() =>
      assertDocumentationLinks(documentationGuide, "no links")
    ).toThrow("no canonical local links");
  });
});

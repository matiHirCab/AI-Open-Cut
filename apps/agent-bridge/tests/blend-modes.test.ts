import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import contract from "../../../contracts/blend-modes-v1.json";
import surface from "../../../contracts/mcp-surface-v1.json";
import { BLEND_MODES } from "../src/blend-modes";
import { blendModeSchema, headlessEditSchema, schemas } from "../src/schemas";
import additions from "./fixtures/blend-mode-additions.json";
import {
  BLEND_PREDECESSOR_PINS,
  projectBlendCatalogPredecessor,
  projectBlendMcpPredecessor,
} from "./fixtures/blend-mode-projection";
import { expandMcpSurfaceCatalog } from "./fixtures/mcp-surface-catalog";

const sorted = (value: unknown): unknown => {
  if (Array.isArray(value)) {
    return value.map(sorted);
  }
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, child]) => [key, sorted(child)])
    );
  }
  return value;
};
it("consumes the canonical closed enum and every malformed selection without fallback", () => {
  expect(BLEND_MODES).toEqual(contract.fields.blendMode.values);
  for (const entry of contract.acceptedSelections) {
    expect(headlessEditSchema.safeParse(entry.operation).success).toBe(true);
    const input: Record<string, unknown> = {
      ...entry.operation,
      expectedRevision: 0,
      projectId: "p",
    };
    Reflect.deleteProperty(input, "operation");
    // The existing MCP adapter requires an actual update; omission preserves
    // blend selection while another authored property changes.
    if (!("blendMode" in input)) {
      input.color = "#ffffff";
    }
    expect(schemas.timelineUpdateItem.safeParse(input).success).toBe(true);
  }
  for (const entry of contract.rejectedSelections) {
    expect(blendModeSchema.safeParse(entry.value).success).toBe(false);
    expect(headlessEditSchema.safeParse(entry.operation).success).toBe(false);
    expect(
      schemas.timelineUpdateItem.safeParse({
        blendMode: entry.value,
        expectedRevision: 0,
        itemId: "visual",
        projectId: "p",
      }).success
    ).toBe(false);
  }
});
it("projects every exact current catalog marker to the verified complete predecessor", () => {
  expect(Object.keys(BLEND_PREDECESSOR_PINS)).toHaveLength(9);
  for (const [name, pin] of Object.entries(BLEND_PREDECESSOR_PINS)) {
    const source = JSON.parse(
      readFileSync(
        new URL(`../../../contracts/${name}.json`, import.meta.url),
        "utf8"
      )
    );
    const before = JSON.stringify(source);
    const projected = projectBlendCatalogPredecessor(name, source);
    expect(
      createHash("sha256")
        .update(JSON.stringify(sorted(projected)))
        .digest("hex"),
      name
    ).toBe(pin);
    expect(JSON.stringify(source)).toBe(before);
    expect(() =>
      projectBlendCatalogPredecessor(name, {
        ...source,
        projectSchemaVersion: 34,
      })
    ).toThrow("Incorrect approved");
    if (name === "extended-visual-animation-v1") {
      expect(() =>
        projectBlendCatalogPredecessor(name, { ...source, unapproved: 42 })
      ).toThrow("Unrelated verified predecessor");
      continue;
    }
    const drift = projectBlendCatalogPredecessor(name, {
      ...source,
      unapproved: 42,
    });
    expect(
      createHash("sha256")
        .update(JSON.stringify(sorted(drift)))
        .digest("hex")
    ).not.toBe(pin);
  }
});
it("preserves the complete verified MCP predecessor and rejects all29 malformed additions without mutation", () => {
  expect(additions.entries).toHaveLength(29);
  const before = JSON.stringify(surface);
  const projected = projectBlendMcpPredecessor(surface);
  expect(
    createHash("sha256")
      .update(JSON.stringify(expandMcpSurfaceCatalog(projected)))
      .digest("hex")
  ).toBe("9d15133960b94806ee92cd9482d10c9481b902f957501fc4742c58b992d73949");
  expect(JSON.stringify(surface)).toBe(before);
  for (const entry of additions.entries) {
    const drift = structuredClone(surface) as unknown as Record<
      string,
      unknown
    >;
    let parent = drift;
    for (const key of entry.path.slice(0, -1)) {
      parent = parent[key] as Record<string, unknown>;
    }
    const key = entry.path.at(-1);
    if (!key) {
      throw new Error("approved addition path missing");
    }
    for (const malformed of [
      null,
      { type: "string" },
      { enum: ["normal", "bogus"], type: "string" },
    ]) {
      parent[key] = malformed;
      const frozen = JSON.stringify(drift);
      expect(() => projectBlendMcpPredecessor(drift)).toThrow(
        "Incorrect approved"
      );
      expect(JSON.stringify(drift)).toBe(frozen);
    }
    Reflect.deleteProperty(parent, key);
    expect(() => projectBlendMcpPredecessor(drift)).toThrow("Missing approved");
  }
});

it("rejects each malformed or missing blend node before unrelated predecessor work", () => {
  for (const entry of additions.entries) {
    for (const missing of [false, true]) {
      const input = structuredClone(surface) as unknown as Record<
        string,
        unknown
      >;
      let parent = input;
      for (const key of entry.path.slice(0, -1)) {
        parent = parent[key] as Record<string, unknown>;
      }
      const key = entry.path.at(-1);
      if (!key) {
        throw new Error("approved addition path missing");
      }
      if (missing) {
        Reflect.deleteProperty(parent, key);
      } else {
        parent[key] = null;
      }
      let unrelatedReads = 0;
      Object.defineProperty(input, "capabilityIdentifiers", {
        get() {
          unrelatedReads += 1;
          throw new Error("Unexpected unrelated predecessor work");
        },
      });
      expect(() => projectBlendMcpPredecessor(input)).toThrow(
        missing ? "Missing approved" : "Incorrect approved"
      );
      expect(unrelatedReads).toBe(0);
      expect(Object.hasOwn(parent, key)).toBe(!missing);
      if (!missing) {
        expect(parent[key]).toBeNull();
      }
    }
  }
});

it("retains unrelated annotation drift through the complete blend predecessor proof", () => {
  const drift = structuredClone(surface);
  drift.toolDefinitions.project_get_state.annotations.readOnlyHint = false;
  const before = JSON.stringify(drift);
  const projected = projectBlendMcpPredecessor(drift);
  expect(projected.capabilityIdentifiers).not.toEqual(
    drift.capabilityIdentifiers
  );
  expect(
    createHash("sha256")
      .update(JSON.stringify(expandMcpSurfaceCatalog(projected)))
      .digest("hex")
  ).not.toBe(
    "9d15133960b94806ee92cd9482d10c9481b902f957501fc4742c58b992d73949"
  );
  expect(JSON.stringify(drift)).toBe(before);
});

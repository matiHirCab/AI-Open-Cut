import { describe, expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const customSkills = [
  "opencut-contract-change",
  "opencut-render-regression",
  "opencut-bridge-verification",
  "opencut-branch",
];

describe("project-scoped OpenCut skills", () => {
  test("have unique discoverable metadata and resolvable local references", () => {
    const descriptions = new Set<string>();

    for (const name of customSkills) {
      const path = resolve(root, ".codex", "skills", name, "SKILL.md");
      const content = readFileSync(path, "utf8");
      const match = content.match(/^---\r?\nname: ([a-z0-9-]+)\r?\ndescription: ([^\r\n]+)\r?\n---\r?\n/);

      expect(match?.[1]).toBe(name);
      expect(match?.[2]?.trim().length).toBeGreaterThan(25);
      expect(descriptions.has(match?.[2] ?? "")).toBe(false);
      descriptions.add(match?.[2] ?? "");

      for (const [, reference] of content.matchAll(/`((?:AGENTS\.md|docs\/[^`]+|openspec\/specs\/[^`]+|contracts\/[^`]+))`/g)) {
        expect(existsSync(resolve(root, reference))).toBe(true);
      }
    }
  });

  test("attributes the Workers skill to the project-local upstream installation", () => {
    const lock = JSON.parse(readFileSync(resolve(root, "skills-lock.json"), "utf8"));
    expect(lock.skills["workers-best-practices"].source).toBe("cloudflare/skills");
    expect(lock.skills["workers-best-practices"].skillPath).toBe(
      "skills/workers-best-practices/SKILL.md"
    );
    expect(
      existsSync(resolve(root, ".agents", "skills", "workers-best-practices", "SKILL.md"))
    ).toBe(true);
  });
});

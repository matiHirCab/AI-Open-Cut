import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import {
  assertPreviewSentinel,
  removePreviewFixtureLink,
  sentinelContents,
  settlePreviewObservation,
  verifyPreviewFixtureControl,
} from "./preview-disposal-controls";

it("retains the primary observation when restoration also fails", async () => {
  const primary = new Error("sentinel lost after removal");
  const secondary = new Error("restoration failed");
  await expect(
    settlePreviewObservation(() => Promise.reject(secondary), primary)
  ).rejects.toMatchObject({ cause: secondary, errors: [primary, secondary] });
});

it("attributes missing and changed sentinels to their observation phase", async () => {
  const root = await mkdtemp(join(tmpdir(), "opencut-sentinel-observation-"));
  const victim = join(root, "outside.png");
  try {
    await expect(
      assertPreviewSentinel(victim, "fixture-only: after removal")
    ).rejects.toThrow("sentinel unreadable: fixture-only: after removal");
    await writeFile(victim, "changed");
    await expect(
      assertPreviewSentinel(victim, "native-only: after disposal")
    ).rejects.toThrow("sentinel changed: native-only: after disposal");
    await writeFile(victim, sentinelContents);
    await expect(
      assertPreviewSentinel(victim, "race: after restoration")
    ).resolves.toBeUndefined();
  } finally {
    await rm(root, { force: true, recursive: true });
  }
});

it("observes twelve fixture replacement cycles without a native process", async () => {
  const phases: string[] = [];
  await verifyPreviewFixtureControl((phase) => phases.push(phase));
  expect(phases.filter((phase) => phase.endsWith(": preserved"))).toHaveLength(
    12
  );
  expect(phases[0]).toBe("fixture-only: cycle 1: before rename");
  expect(phases.at(-1)).toBe("fixture-only: cycle 12: preserved");
});

it("recovery removes only the fixture link and refuses to remove a directory", async () => {
  const root = await mkdtemp(join(tmpdir(), "opencut-link-recovery-"));
  const outside = join(root, "outside");
  const link = join(root, "link");
  const victim = join(outside, "victim");
  try {
    await mkdir(outside);
    await writeFile(victim, sentinelContents);
    await symlink(
      outside,
      link,
      process.platform === "win32" ? "junction" : "dir"
    );
    await removePreviewFixtureLink(link);
    await expect(readFile(link)).rejects.toMatchObject({ code: "ENOENT" });
    await removePreviewFixtureLink(outside);
    await assertPreviewSentinel(victim, "recovery: target directory preserved");
    await removePreviewFixtureLink(link);
  } finally {
    await removePreviewFixtureLink(link);
    await rm(root, { force: true, recursive: true });
  }
});

import { randomUUID } from "node:crypto";
import {
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  rename,
  rm,
  rmdir,
  symlink,
  unlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import { expect } from "vitest";

export const sentinelContents = "outside must survive";

export const assertPreviewSentinel = async (path: string, phase: string) => {
  let contents: string;
  try {
    contents = await readFile(path, "utf8");
  } catch (cause) {
    throw new Error(`Preview disposal sentinel unreadable: ${phase}`, {
      cause,
    });
  }
  expect(contents, `Preview disposal sentinel changed: ${phase}`).toBe(
    sentinelContents
  );
};

// Recovery only: unlink an owned fixture link before recursively removing its
// temporary tree. This does not replace the teardown operation under diagnosis.
export const removePreviewFixtureLink = async (path: string) => {
  const metadata = await lstat(path).catch((error: NodeJS.ErrnoException) => {
    if (error.code === "ENOENT") {
      return;
    }
    throw error;
  });
  if (metadata?.isSymbolicLink()) {
    if (process.platform === "win32") {
      await rmdir(path);
    } else {
      await unlink(path);
    }
  }
};

export const settlePreviewObservation = async (
  action: () => Promise<void>,
  priorFailure: unknown
) => {
  try {
    await action();
  } catch (cause) {
    if (priorFailure) {
      // biome-ignore lint/style/useErrorCause: AggregateError receives ErrorOptions as its third argument.
      throw new AggregateError(
        [priorFailure, cause],
        "Preview disposal observation and restoration failed",
        { cause }
      );
    }
    throw cause;
  }
};

export const cleanupPreviewFixture = async (
  root: string,
  links: string[],
  failed: boolean,
  observe: (phase: string) => void
) => {
  try {
    for (const path of links) {
      // biome-ignore lint/performance/noAwaitInLoops: Remove inner owned link before its ancestor.
      await removePreviewFixtureLink(path);
    }
    await rm(root, { force: true, recursive: true });
  } catch (error) {
    observe("recovery cleanup failed");
    if (!failed) {
      throw error;
    }
  }
};

export const verifyPreviewFixtureControl = async (
  observe: (phase: string) => void
) => {
  const root = await mkdtemp(
    join(tmpdir(), "opencut-disposal-fixture-control-")
  );
  const previews = join(root, "projects", randomUUID(), "previews");
  const moved = `${previews}-saved`;
  const victim = join(root, `preview-${randomUUID()}.png`);
  let failed = false;
  try {
    await mkdir(previews, { recursive: true });
    await writeFile(victim, sentinelContents);
    await writeFile(join(previews, basename(victim)), "safe");
    await assertPreviewSentinel(victim, "fixture-only: before replacement");
    for (let attempt = 0; attempt < 12; attempt += 1) {
      const phase = `fixture-only: cycle ${attempt + 1}`;
      observe(`${phase}: before rename`);
      // biome-ignore lint/performance/noAwaitInLoops: Observe each real filesystem phase independently.
      await rename(previews, moved);
      await symlink(
        root,
        previews,
        process.platform === "win32" ? "junction" : "dir"
      );
      await assertPreviewSentinel(victim, `${phase}: after link creation`);
      observe(`${phase}: before recursive link removal`);
      await rm(previews, { maxRetries: 50, recursive: true, retryDelay: 2 });
      await assertPreviewSentinel(
        victim,
        `${phase}: after recursive link removal`
      );
      await rename(moved, previews);
      await assertPreviewSentinel(victim, `${phase}: after restoration`);
      expect(await readFile(join(previews, basename(victim)), "utf8")).toBe(
        "safe"
      );
      observe(`${phase}: preserved`);
    }
  } catch (error) {
    failed = true;
    throw error;
  } finally {
    await cleanupPreviewFixture(root, [previews], failed, observe);
  }
};

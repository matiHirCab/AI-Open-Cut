import { mock } from "bun:test";

const filesystem = await import("node:fs/promises");
const remove = filesystem.rm;
mock.module("node:fs/promises", () => ({
  ...filesystem,
  rm: (path, options) => {
    if (
      typeof path === "string" &&
      path.includes(".opencut-doctor-") &&
      path.endsWith(".tmp")
    ) {
      return Promise.reject(new Error("Private probe deletion failed"));
    }
    return remove(path, options);
  },
}));

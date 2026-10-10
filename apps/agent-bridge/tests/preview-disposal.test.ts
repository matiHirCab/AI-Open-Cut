import { randomUUID } from "node:crypto";
import { expect, it } from "vitest";
import { loadBridgeConfig } from "../src/config";
import { previewDisposer } from "../src/preview-disposal";
import type { Job } from "../src/schemas";

const job: Job = {
  artifact: {
    mimeType: "image/png",
    relativePath: `previews/preview-${randomUUID()}.png`,
    sizeBytes: 4,
    warnings: [],
  },
  createdAtMs: 0,
  expiresAtMs: 10,
  jobId: randomUUID(),
  kind: "preview",
  persistence: "process",
  progress: 1,
  projectId: randomUUID(),
  revision: 1,
  status: "completed",
  updatedAtMs: 0,
};
const dispose = (script: string, timeout = 1000) =>
  previewDisposer({
    ...loadBridgeConfig({
      ...process.env,
      OPENCUT_PROJECTS_DIR: "/private/path",
    }),
    headlessArguments: ["-e", script, "--"],
    headlessPath: process.execPath,
    headlessRequestTimeoutMs: timeout,
  });
it("accepts only bounded private success after process termination", async () => {
  await expect(
    dispose(
      'process.stdin.resume(); process.stdin.on("end",()=>console.log(JSON.stringify({type:"result",result:{disposed:true}})))'
    )(job)
  ).resolves.toBeUndefined();
});
it.each([
  'console.log("secret"); process.exit(1)',
  'console.log("x".repeat(2048))',
  'console.error("private path")',
  'console.log(JSON.stringify({type:"result",result:{disposed:true}})); process.exit(1)',
  'console.log("{}")',
])("fails closed and redacts malformed cleanup", async (script) => {
  await expect(dispose(script)(job)).rejects.toMatchObject({
    code: "VALIDATION_FAILED",
    message: "Preview artifact disposal is unavailable or unsafe",
  });
});
it("terminates hung cleanup without releasing ownership", async () => {
  await expect(
    dispose("setInterval(()=>{}, 1000)", 50)(job)
  ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
});
it("rejects invalid owned descriptors before dispatch", async () => {
  await expect(
    dispose("process.exit(0)")({
      ...job,
      artifact: {
        mimeType: "image/png",
        relativePath: "../../foreign",
        sizeBytes: 4,
        warnings: [],
      },
    })
  ).rejects.toMatchObject({ code: "VALIDATION_FAILED" });
});

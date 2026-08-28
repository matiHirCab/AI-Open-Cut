import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

process.stdin.setEncoding("utf8");
let input = "";
for await (const chunk of process.stdin) {
  input += chunk;
}
const request = JSON.parse(input) as {
  operation: string;
  projectId?: string;
};

if (request.operation === "hang") {
  setInterval(() => undefined, 1000);
} else if (request.operation === "render_preview") {
  const projects = process.env.OPENCUT_PROJECTS_DIR;
  const requestId = process.env.OPENCUT_REQUEST_ID;
  if (projects && request.projectId && requestId) {
    const previews = join(projects, request.projectId, "previews");
    mkdirSync(previews, { recursive: true });
    writeFileSync(join(previews, `.opencut-${requestId}.png`), "partial");
  }
  setInterval(() => undefined, 1000);
} else if (request.operation === "malformed") {
  process.stdout.write("not-json\n");
} else if (request.operation === "partial") {
  process.stdout.write('{"type":"res');
  setTimeout(() => {
    process.stdout.write('ult","result":{"ok":true}}\n');
  }, 10);
} else {
  process.exitCode = 2;
}

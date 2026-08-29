import { createHmac, timingSafeEqual } from "node:crypto";
import {
  createServer as createHttpServer,
  type IncomingMessage,
  type ServerResponse,
} from "node:http";

import { toNodeHandler } from "@modelcontextprotocol/node";
import { createMcpHandler } from "@modelcontextprotocol/server";

import type { BridgeConfig } from "./config";
import type { HeadlessClient } from "./headless";
import type { JobRegistry } from "./jobs";
import { createServer } from "./server";
import type { SpeechApplicationService } from "./speech";
import type { TranscriptionApplicationService } from "./transcription";

const json = (response: ServerResponse, status: number, body: unknown) => {
  response.writeHead(status, {
    "cache-control": "no-store",
    "content-type": "application/json; charset=utf-8",
  });
  response.end(JSON.stringify(body));
};

const tokenMatches = (provided: string, expected: string) => {
  const left = createHmac("sha256", "opencut-http-auth")
    .update(provided)
    .digest();
  const right = createHmac("sha256", "opencut-http-auth")
    .update(expected)
    .digest();
  return timingSafeEqual(left, right);
};

const readJsonBody = async (request: IncomingMessage, maximum: number) => {
  const chunks: Buffer[] = [];
  let size = 0;
  for await (const chunk of request) {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    size += buffer.length;
    if (size > maximum) {
      throw new Error("BODY_TOO_LARGE");
    }
    chunks.push(buffer);
  }
  if (size === 0) {
    return;
  }
  return JSON.parse(Buffer.concat(chunks).toString("utf8")) as unknown;
};

export const serveHttp = (
  speech: SpeechApplicationService,
  transcription: TranscriptionApplicationService,
  headless: HeadlessClient,
  jobs: JobRegistry,
  config: BridgeConfig,
  health: () => Promise<Record<string, unknown>>
) => {
  const handler = createMcpHandler(() =>
    createServer(speech, headless, jobs, config, transcription)
  );
  const nodeHandler = toNodeHandler(handler);
  // biome-ignore lint/complexity/noExcessiveCognitiveComplexity: one boundary handler keeps all rejection paths before MCP dispatch.
  const listener = createHttpServer(async (request, response) => {
    const {
      authorization,
      host,
      origin,
      "content-length": contentLength,
    } = request.headers;
    if (
      !(
        host &&
        (host === config.httpHost ||
          host === `${config.httpHost}:${config.httpPort}` ||
          (config.httpHost === "::1" &&
            ["[::1]", `[::1]:${config.httpPort}`].includes(host)))
      )
    ) {
      json(response, 403, { error: "invalid host" });
      return;
    }
    if (origin && !config.httpAllowedOrigins.includes(origin)) {
      json(response, 403, { error: "invalid origin" });
      return;
    }
    if (origin) {
      response.setHeader("access-control-allow-origin", origin);
      response.setHeader("vary", "Origin");
    }
    const path = new URL(request.url ?? "/", `http://${host}`).pathname;
    if (request.method === "OPTIONS" && path === "/mcp" && origin) {
      response.writeHead(204, {
        "access-control-allow-headers":
          "authorization, content-type, mcp-protocol-version, mcp-session-id",
        "access-control-allow-methods": "GET, POST, DELETE, OPTIONS",
        "access-control-max-age": "600",
      });
      response.end();
      return;
    }
    if (config.httpAuthToken) {
      const header = authorization ?? "";
      const provided = header.startsWith("Bearer ") ? header.slice(7) : "";
      if (!tokenMatches(provided, config.httpAuthToken)) {
        response.setHeader("www-authenticate", "Bearer");
        json(response, 401, { error: "unauthorized" });
        return;
      }
    }
    if (path === "/health" && request.method === "GET") {
      json(response, 200, await health());
      return;
    }
    if (path !== "/mcp") {
      json(response, 404, { error: "not found" });
      return;
    }
    try {
      const length = Number(contentLength ?? 0);
      if (Number.isFinite(length) && length > config.httpMaxBodyBytes) {
        throw new Error("BODY_TOO_LARGE");
      }
      const parsedBody =
        request.method === "POST"
          ? await readJsonBody(request, config.httpMaxBodyBytes)
          : undefined;
      await nodeHandler(
        request as unknown as Parameters<typeof nodeHandler>[0],
        response,
        parsedBody
      );
    } catch (error) {
      if (response.headersSent) {
        response.end();
      } else {
        json(
          response,
          error instanceof Error && error.message === "BODY_TOO_LARGE"
            ? 413
            : 400,
          { error: "invalid request" }
        );
      }
    }
  });
  listener.listen(config.httpPort, config.httpHost);
  return {
    close: async () => {
      await Promise.race([
        new Promise<void>((resolve, reject) => {
          listener.close((error) => (error ? reject(error) : resolve()));
          listener.closeAllConnections();
        }),
        new Promise<void>((resolve) => setTimeout(resolve, 2000)),
      ]);
      await Promise.race([
        handler.close(),
        new Promise<void>((resolve) => setTimeout(resolve, 2000)),
      ]);
    },
  };
};

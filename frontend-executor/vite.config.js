import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import crypto from "node:crypto";
import fs from "node:fs";
import http from "node:http";
import path from "node:path";

function sessionKeyPath() {
  if (process.env.QUANTPILOT_SESSION_KEY_PATH) return process.env.QUANTPILOT_SESSION_KEY_PATH;
  const storageRoot = process.env.QUANTPILOT_STORAGE_ROOT || path.resolve(process.cwd(), "..", "storage");
  return path.join(storageRoot, ".executor-session-key");
}

function signExecutorRequest(method, pathAndQuery, timestampMs, body) {
  const encoded = fs.readFileSync(sessionKeyPath(), "utf8").trim();
  const key = Buffer.from(encoded, "base64");
  const payloadPrefix = Buffer.from(`${method.toUpperCase()}\n${pathAndQuery}\n${timestampMs}\n`, "utf8");
  return crypto.createHmac("sha256", key).update(Buffer.concat([payloadPrefix, body])).digest("base64");
}

function signedExecutorProxy() {
  return {
    name: "quantpilot-signed-executor-proxy",
    configureServer(server) {
      server.middlewares.use("/api/executor", (req, res, next) => {
        const chunks = [];
        req.on("data", (chunk) => chunks.push(chunk));
        req.on("error", next);
        req.on("end", () => {
          const body = Buffer.concat(chunks);
          const pathAndQuery = `/api/executor${req.url || ""}`;
          const timestampMs = Date.now().toString();
          let signature;
          try {
            signature = signExecutorRequest(req.method || "GET", pathAndQuery, timestampMs, body);
          } catch (error) {
            res.statusCode = 503;
            res.setHeader("content-type", "application/json");
            res.end(JSON.stringify({
              error: "executor_session_key_unavailable",
              message: `执行端会话密钥不可用: ${error.message}`,
            }));
            return;
          }

          const headers = {
            ...req.headers,
            host: "127.0.0.1:3001",
            "content-length": String(body.length),
            "x-executor-timestamp": timestampMs,
            "x-executor-signature": signature,
          };

          const proxyReq = http.request(
            {
              hostname: "127.0.0.1",
              port: 3001,
              method: req.method,
              path: pathAndQuery,
              headers,
            },
            (proxyRes) => {
              res.writeHead(proxyRes.statusCode || 502, proxyRes.headers);
              proxyRes.pipe(res);
            },
          );
          proxyReq.on("error", (error) => {
            res.statusCode = 502;
            res.setHeader("content-type", "application/json");
            res.end(JSON.stringify({
              error: "executor_proxy_failed",
              message: error.message,
            }));
          });
          if (body.length > 0) proxyReq.write(body);
          proxyReq.end();
        });
      });
    },
  };
}

export default defineConfig({
  plugins: [react(), signedExecutorProxy()],
  server: {
    port: 5174,
  },
  build: {
    outDir: "dist",
  },
});

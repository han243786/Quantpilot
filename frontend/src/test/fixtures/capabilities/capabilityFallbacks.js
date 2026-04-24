import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const fixtureDir = path.dirname(fileURLToPath(import.meta.url));
const backendCapabilityFixturePath = path.join(fixtureDir, "backend-capabilities-v1.json");

export const backendCapabilitiesFixture = JSON.parse(
  fs.readFileSync(backendCapabilityFixturePath, "utf8")
);

export const capabilityFallbackFixtures = {
  cacheKey: "quantpilot_capabilities_cache",
  serviceUnavailableText: "capability service unavailable",
  serviceUnavailableHttp503: {
    status: 503,
    contentType: "text/plain; charset=utf-8",
    body: "capability service unavailable"
  }
};

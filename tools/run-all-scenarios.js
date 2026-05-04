#!/usr/bin/env node
// Cross-platform scenario test runner
const { execSync, spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

const ROOT = path.resolve(__dirname, "..");
const SCENARIO_DIR = path.join(ROOT, "tests", "scenarios");

function log(msg) {
  console.log(`[${new Date().toISOString().slice(11, 19)}] ${msg}`);
}

async function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function main() {
  log("Starting QuantPilot backend...");
  const backendBin = path.join(ROOT, "target", "debug", "quantpilot.exe");
  if (!fs.existsSync(backendBin)) {
    console.error("Backend binary not found at:", backendBin);
    console.error("Run: cargo build");
    process.exit(1);
  }

  const backend = spawn(backendBin, [], {
    env: { ...process.env, QUANTPILOT_PORT: "3000", QUANTPILOT_DEV: "true", QUANTPILOT_API_KEY: "" },
    stdio: "pipe",
  });

  await sleep(4000);

  // Verify backend
  try {
    await fetch("http://127.0.0.1:3000/api/health");
    log("Backend ready");
  } catch {
    log("Backend not responding, waiting...");
    await sleep(5000);
  }

  log("Running scenarios...");
  const files = fs.readdirSync(SCENARIO_DIR)
    .filter((f) => f.endsWith(".qs") && f.startsWith("scenario_"))
    .sort();

  let passed = 0;
  let failed = 0;
  const runScenario = path.join(ROOT, "tools", "run-scenario.js");

  for (const f of files) {
    const fullPath = path.join(SCENARIO_DIR, f);
    try {
      execSync(`node "${runScenario}" "${fullPath}"`, { stdio: "inherit" });
      passed++;
    } catch {
      failed++;
      console.error(`  FAILED: ${f}`);
    }
  }

  log(`Done: ${passed} passed, ${failed} failed`);

  // Cleanup backend
  backend.kill();
  await sleep(1000);
  process.exit(failed > 0 ? 1 : 0);
}

main();

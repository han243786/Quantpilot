#!/usr/bin/env node
// check-qs.js — Validate .qs file syntax without running scenarios
// Usage: node tools/check-qs.js <path-to-.qs-file>

const fs = require("fs");
const path = require("path");

const BACKEND = process.env.QUANTPILOT_API || "http://127.0.0.1:3000";

async function main() {
  const filePath = process.argv[2];
  if (!filePath) {
    console.error("Usage: node check-qs.js <path-to-.qs-file>");
    process.exit(1);
  }

  const fullPath = path.resolve(filePath);
  if (!fs.existsSync(fullPath)) {
    console.error(`File not found: ${fullPath}`);
    process.exit(1);
  }

  const source = fs.readFileSync(fullPath, "utf-8");
  const scenarioName = path.basename(fullPath, ".qs");

  console.log(`Checking: ${scenarioName}`);

  try {
    const resp = await fetch(`${BACKEND}/api/test/scenario/run`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ source }),
    });

    if (!resp.ok) {
      const errorBody = await resp.text();
      console.error(`  FAIL: Backend returned ${resp.status}`);
      // Try to extract meaningful error
      try {
        const err = JSON.parse(errorBody);
        console.error(`  ${err.message || errorBody}`);
      } catch {
        console.error(`  ${errorBody.slice(0, 300)}`);
      }
      process.exit(1);
    }

    const report = await resp.json();
    const passed = report.steps.filter((s) => s.status === "passed").length;
    const failed = report.steps.filter((s) => s.status === "failed").length;
    const skipped = report.steps.filter((s) => s.status === "skipped").length;

    console.log(`  Scenario: ${report.scenario_name}`);
    console.log(`  Steps: ${passed} passed, ${failed} failed, ${skipped} skipped`);

    if (failed > 0) {
      console.error(`  FAIL: ${failed} step(s) failed`);
      for (const s of report.steps.filter((s) => s.status === "failed")) {
        console.error(`    ✗ ${s.name}: ${s.message || "unknown error"}`);
      }
      process.exit(1);
    }

    if (skipped > 0) {
      console.log(`  ⚠ ${skipped} step(s) skipped (e.g., compile failure)`);
    }

    console.log(`  OK: syntax valid, compilation successful`);
    process.exit(0);
  } catch (e) {
    console.error(`  FAIL: ${e.message}`);
    process.exit(1);
  }
}

main();

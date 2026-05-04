#!/usr/bin/env node
// run-scenario.js — Run a .qs scenario file against the backend and print results
// Usage: node tools/run-scenario.js tests/scenarios/scenario_01.qs

const fs = require("fs");
const path = require("path");

const BACKEND = process.env.QUANTPILOT_API || "http://127.0.0.1:3000";
const REPORT_DIR = path.resolve(__dirname, "..", "markdown", "测试", "test-reports");

async function main() {
  const filePath = process.argv[2];
  if (!filePath) {
    console.error("Usage: node run-scenario.js <path-to-.qs-file>");
    process.exit(1);
  }

  const fullPath = path.resolve(filePath);
  if (!fs.existsSync(fullPath)) {
    console.error(`File not found: ${fullPath}`);
    process.exit(1);
  }

  const source = fs.readFileSync(fullPath, "utf-8");
  const scenarioName = path.basename(fullPath, ".qs");

  console.log(`\n${"=".repeat(60)}`);
  console.log(`Running: ${scenarioName}`);
  console.log(`${"=".repeat(60)}`);

  try {
    const resp = await fetch(`${BACKEND}/api/test/scenario/run`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ source }),
    });

    if (!resp.ok) {
      const errorBody = await resp.text();
      console.error(`Backend returned ${resp.status}: ${errorBody.slice(0, 500)}`);
      process.exit(1);
    }

    const report = await resp.json();

    // Print results
    console.log(`Scenario: ${report.scenario_name}`);
    console.log(`Cover: ${(report.cover || []).join(", ")}`);
    console.log(`Duration: ${report.duration_ms}ms\n`);

    let exitCode = 0;
    for (const step of report.steps) {
      const icon = step.status === "passed" ? "✓" : step.status === "failed" ? "✗" : "⊘";
      if (step.status === "failed") exitCode = 1;
      console.log(`  ${icon} ${step.name} (${step.duration_ms}ms)`);
      if (step.message) {
        console.log(`    ${step.message}`);
      }
      if (step.data_snapshot) {
        const keys = Object.keys(step.data_snapshot).join(", ");
        console.log(`    snapshot: {${keys}}`);
      }
    }

    const passed = report.steps.filter((s) => s.status === "passed").length;
    const total = report.steps.length;
    console.log(`\n${passed}/${total} passed`);

    // Save report JSON
    if (!fs.existsSync(REPORT_DIR)) {
      fs.mkdirSync(REPORT_DIR, { recursive: true });
    }
    const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
    const reportPath = path.join(REPORT_DIR, `${scenarioName}-${timestamp}.json`);
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    console.log(`Report saved: ${reportPath}`);

    process.exit(exitCode);
  } catch (e) {
    console.error(`Error: ${e.message}`);
    process.exit(1);
  }
}

main();

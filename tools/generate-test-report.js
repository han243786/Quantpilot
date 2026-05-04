#!/usr/bin/env node
// generate-test-report.js — Merge scenario reports + Playwright results into markdown
// Usage: node tools/generate-test-report.js

const fs = require("fs");
const path = require("path");

const REPORT_DIR = path.resolve(__dirname, "..", "markdown", "测试", "test-reports");
const OUTPUT_PATH = path.resolve(__dirname, "..", "markdown", "测试", "测试报告-latest.md");
const PLAYWRIGHT_DIR = path.resolve(__dirname, "..", "frontend", "test-results");

function loadScenarioReports() {
  if (!fs.existsSync(REPORT_DIR)) return [];
  const files = fs.readdirSync(REPORT_DIR).filter((f) => f.endsWith(".json"));
  if (files.length === 0) return [];

  // Group by scenario name (take latest timestamp for each)
  const latest = new Map();
  for (const f of files) {
    const base = f.split("-").slice(0, -5).join("-");
    if (!latest.has(base) || f > latest.get(base)) {
      latest.set(base, f);
    }
  }

  return Array.from(latest.values())
    .map((f) => {
      try {
        return JSON.parse(fs.readFileSync(path.join(REPORT_DIR, f), "utf-8"));
      } catch {
        return null;
      }
    })
    .filter(Boolean);
}

function loadPlaywrightResults() {
  const jsonPath = path.join(PLAYWRIGHT_DIR, ".last-run.json");
  if (!fs.existsSync(jsonPath)) return null;
  try {
    return JSON.parse(fs.readFileSync(jsonPath, "utf-8"));
  } catch {
    return null;
  }
}

function generateMarkdown(scenarios, pwResult) {
  const now = new Date().toISOString().slice(0, 19).replace("T", " ");
  let md = `# QuantPilot 测试报告\n\n> 生成时间: ${now}\n\n`;

  // ── Summary ──
  md += `## 总览\n\n`;
  md += `| 类型 | 通过 | 失败 | 跳过 |\n`;
  md += `|------|:----:|:----:|:----:|\n`;

  let totalPassed = 0;
  let totalFailed = 0;
  let totalSkipped = 0;
  for (const s of scenarios) {
    totalPassed += s.passed_count || 0;
    totalFailed += s.failed_count || 0;
    totalSkipped += s.skipped_count || 0;
  }
  const total = totalPassed + totalFailed + totalSkipped;
  md += `| 后端场景 | ${totalPassed} | ${totalFailed} | ${totalSkipped} |\n`;

  if (pwResult) {
    const pwPassed = pwResult.passed || 0;
    const pwFailed = pwResult.failed || 0;
    md += `| 前端 E2E | ${pwPassed} | ${pwFailed} | 0 |\n`;
  }

  md += `\n`;

  // ── Backend Scenarios ──
  md += `## 后端场景\n\n`;
  for (const s of scenarios) {
    const passed = s.passed_count || 0;
    const totalSteps = s.steps.length;
    const icon = s.failed_count > 0 ? "⚠️" : "✅";
    md += `### ${icon} ${s.scenario_name}\n\n`;
    md += `- **通过**: ${passed}/${totalSteps}\n`;
    md += `- **耗时**: ${s.duration_ms}ms\n`;
    md += `- **覆盖**: ${(s.cover || []).join(", ") || "—"}\n\n`;

    md += `| 步骤 | 状态 | 耗时 |\n`;
    md += `|------|:----:|-----|\n`;
    for (const step of s.steps) {
      const status = step.status === "passed" ? "✓" : step.status === "failed" ? "✗" : "⊘";
      md += `| ${step.name} | ${status} | ${step.duration_ms}ms |\n`;
    }
    md += `\n`;
  }

  // ── No results ──
  if (scenarios.length === 0) {
    md += `> ⚠️ 未找到后端场景报告。请先运行 \`node tools/run-scenario.js\`。\n\n`;
  }

  return md;
}

function main() {
  const scenarios = loadScenarioReports();
  const pwResult = loadPlaywrightResults();
  const md = generateMarkdown(scenarios, pwResult);

  const dir = path.dirname(OUTPUT_PATH);
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(OUTPUT_PATH, md);

  console.log(md);
  console.log(`\nReport saved: ${OUTPUT_PATH}`);
}

main();

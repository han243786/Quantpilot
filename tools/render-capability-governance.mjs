import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { CAPABILITY_GOVERNANCE } from "../frontend/src/capabilities/capabilityGovernance.js";
import {
  DECLARED_INDICATOR_KINDS,
  SUPPORTED_EXCHANGES,
  SUPPORTED_FRONTEND_MODULE_KEYS,
  SUPPORTED_RUNTIME_EXECUTION_MODULES,
  SUPPORTED_RUNTIME_MODES,
  SUPPORTED_SYMBOLS
} from "../frontend/src/capabilities/supportMatrix.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..");

function normalizeNewlines(value) {
  return value.replace(/\r\n/g, "\n");
}

function escapeCell(value) {
  return String(value ?? "")
    .replace(/\|/g, "\\|")
    .replace(/\r?\n/g, "<br>");
}

function renderTable(headers, rows) {
  const headerLine = `| ${headers.join(" | ")} |`;
  const separatorLine = `| ${headers.map(() => "---").join(" | ")} |`;
  const bodyLines = rows.map((row) => `| ${row.map(escapeCell).join(" | ")} |`);
  return [headerLine, separatorLine, ...bodyLines].join("\n");
}

function summarizeBy(items, keyFn) {
  const counts = new Map();
  for (const item of items) {
    const key = keyFn(item);
    counts.set(key, (counts.get(key) || 0) + 1);
  }
  return Array.from(counts.entries()).sort(([left], [right]) => left.localeCompare(right));
}

const classLabels = new Map([
  ["supported", "supported（已支持）"],
  ["restricted", "restricted（受限）"],
  ["trace_only", "trace_only（仅追踪）"],
  ["disallowed_claim", "disallowed_claim（禁止声明）"]
]);

const familyLabels = new Map([
  ["runtime_mode", "runtime_mode（运行模式）"],
  ["execution_module", "execution_module（执行模块）"],
  ["exchange", "exchange（交易所）"],
  ["symbol", "symbol（交易对）"],
  ["strategy_ir_indicator_kind", "strategy_ir_indicator_kind（策略 IR 指标类型）"],
  ["frontend_module", "frontend_module（前端模块）"],
  ["ui_action", "ui_action（UI 操作）"],
  ["workspace_surface", "workspace_surface（工作区界面）"],
  ["compile_boundary", "compile_boundary（编译边界）"],
  ["user_facing_claim", "user_facing_claim（面向用户声明）"]
]);

function renderClassLabel(className) {
  return classLabels.get(className) || className;
}

function renderFamilyLabel(family) {
  return familyLabels.get(family) || family;
}

function renderRegistrySnapshot() {
  const registry = CAPABILITY_GOVERNANCE.registry;
  const classSummary = summarizeBy(registry, (entry) => entry.class);
  const familySummary = summarizeBy(registry, (entry) => entry.family);
  const familyOrder = [...new Set(registry.map((entry) => entry.family))];

  const lines = [
    "# 生成的能力治理注册表",
    "",
    "此文件由 `frontend/src/capabilities/capabilityGovernance.js` 生成。",
    "请勿手动编辑。",
    "",
    `模式版本：\`${CAPABILITY_GOVERNANCE.schemaVersion}\``,
    "",
    "使用以下命令重新生成此快照：",
    "",
    "```powershell",
    "powershell -NoProfile -ExecutionPolicy Bypass -File tools\\check-capability-governance.ps1 -WriteSnapshot",
    "```",
    "",
    "## 按类别汇总",
    "",
    renderTable(
      ["类别", "条目数"],
      classSummary.map(([className, count]) => [renderClassLabel(className), count])
    ),
    "",
    "## 按系列汇总",
    "",
    renderTable(
      ["系列", "条目数"],
      familySummary.map(([family, count]) => [renderFamilyLabel(family), count])
    )
  ];

  for (const family of familyOrder) {
    const familyEntries = registry.filter((entry) => entry.family === family);
    lines.push(
      "",
      `## ${renderFamilyLabel(family)}`,
      "",
      renderTable(
        ["ID", "值", "类别", "负责人角色", "审查责任", "真实数据源", "备注"],
        familyEntries.map((entry) => [
          entry.id,
          entry.value,
          entry.class,
          entry.ownerRole,
          entry.reviewResponsibility,
          entry.sourceOfTruth,
          (entry.notes || []).join("; ")
        ])
      )
    );
  }

  lines.push("");
  return lines.join("\n");
}

function renderTextGatePayload() {
  const claimEntries = CAPABILITY_GOVERNANCE.registry.filter(
    (entry) => entry.family === "user_facing_claim"
  );
  const allowedClaims = claimEntries
    .filter((entry) => entry.class === "supported")
    .map((entry) => ({
      id: entry.id,
      value: entry.value,
      approvedPhrase: entry.textGate?.approvedPhrase || entry.value
    }));
  const disallowedClaims = claimEntries
    .filter((entry) => entry.class === "disallowed_claim")
    .map((entry) => {
      if (!entry.textGate?.forbiddenPattern || !entry.textGate?.detail) {
        throw new Error(`Disallowed claim is missing text-gate metadata: ${entry.id}`);
      }

      return {
        id: entry.id,
        value: entry.value,
        forbiddenPattern: entry.textGate.forbiddenPattern,
        detail: entry.textGate.detail,
        allowedContextPattern: entry.textGate.allowedContextPattern || ""
      };
    });

  return JSON.stringify(
    {
      schemaVersion: CAPABILITY_GOVERNANCE.schemaVersion,
      positiveClaimAudit: CAPABILITY_GOVERNANCE.textGates.positiveClaimAudit,
      allowedClaims,
      disallowedClaims
    },
    null,
    2
  );
}

function assertArrayEqual(name, actual, expected) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(`${name} drift detected. expected=${expectedJson} actual=${actualJson}`);
  }
}

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, relativePath), "utf8"));
}

function readUtf8(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function assertOpenApiCapabilityResponseAligned() {
  const openapi = normalizeNewlines(readUtf8("contracts/openapi/root.yaml"));
  const start = openapi.indexOf("    CapabilityResponse:\n");
  const end = openapi.indexOf("    AiProposalRecord:\n", start);
  if (start < 0 || end < 0 || end <= start) {
    throw new Error("OpenAPI CapabilityResponse schema block is missing.");
  }

  const block = openapi.slice(start, end);
  const requiredFields = [
    "api_version",
    "schema_version",
    "schema_hash",
    "chain_stages",
    "strategy_ir",
    "runtime",
    "market_data",
    "frontend",
    "versioning",
    "permission_boundary"
  ];

  for (const field of requiredFields) {
    if (!block.includes(`        ${field}:`)) {
      throw new Error(`OpenAPI CapabilityResponse schema is missing property: ${field}`);
    }
    if (!block.includes(field)) {
      throw new Error(`OpenAPI CapabilityResponse schema is missing required field: ${field}`);
    }
  }

  for (const enumValue of ["supported", "declared_only", "proposal_only", "disabled", "deny", "allow"]) {
    if (!block.includes(enumValue)) {
      throw new Error(`OpenAPI CapabilityResponse schema is missing enum value: ${enumValue}`);
    }
  }
}

function assertSupportMatrixAligned() {
  const fixture = readJson("frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json");
  assertArrayEqual("runtime.supported_modes", fixture.runtime.supported_modes, SUPPORTED_RUNTIME_MODES);
  assertArrayEqual(
    "runtime.supported_execution_modules",
    fixture.runtime.supported_execution_modules,
    SUPPORTED_RUNTIME_EXECUTION_MODULES
  );
  assertArrayEqual("market_data.supported_exchanges", fixture.market_data.supported_exchanges, SUPPORTED_EXCHANGES);
  assertArrayEqual("market_data.supported_symbols", fixture.market_data.supported_symbols, SUPPORTED_SYMBOLS);
  assertArrayEqual(
    "strategy_ir.declared_indicator_kinds",
    fixture.strategy_ir.declared_indicator_kinds,
    DECLARED_INDICATOR_KINDS
  );
  assertArrayEqual(
    "strategy_ir.supported_indicator_kinds",
    fixture.strategy_ir.supported_indicator_kinds,
    DECLARED_INDICATOR_KINDS
  );
  assertArrayEqual(
    "frontend.supported_module_keys",
    fixture.frontend.supported_module_keys,
    SUPPORTED_FRONTEND_MODULE_KEYS
  );
}

function assertCapabilityContractAlignment() {
  assertSupportMatrixAligned();
  assertOpenApiCapabilityResponseAligned();
}

function resolveOutputPath(targetPath) {
  if (!targetPath) {
    throw new Error("Missing output path.");
  }
  return path.isAbsolute(targetPath) ? targetPath : path.join(repoRoot, targetPath);
}

function writeSnapshot(targetPath) {
  assertCapabilityContractAlignment();
  const resolvedPath = resolveOutputPath(targetPath);
  fs.mkdirSync(path.dirname(resolvedPath), { recursive: true });
  fs.writeFileSync(resolvedPath, renderRegistrySnapshot(), "utf8");
  process.stdout.write(`Capability governance snapshot written to ${resolvedPath}\n`);
}

function checkSnapshot(targetPath) {
  assertCapabilityContractAlignment();
  const resolvedPath = resolveOutputPath(targetPath);
  if (!fs.existsSync(resolvedPath)) {
    process.stderr.write(
      `Capability governance snapshot is missing at ${resolvedPath}. Run the write command first.\n`
    );
    process.exit(1);
  }

  const expected = normalizeNewlines(renderRegistrySnapshot());
  const actual = normalizeNewlines(fs.readFileSync(resolvedPath, "utf8"));

  if (actual !== expected) {
    process.stderr.write(
      `Capability governance snapshot drift detected at ${resolvedPath}.\n` +
        "Run `powershell -NoProfile -ExecutionPolicy Bypass -File tools\\check-capability-governance.ps1 -WriteSnapshot` to update it.\n"
    );
    process.exit(1);
  }

  process.stdout.write(`Capability governance snapshot is up to date: ${resolvedPath}\n`);
}

const [mode, targetPath] = process.argv.slice(2);

if (mode === "--write") {
  writeSnapshot(targetPath);
} else if (mode === "--check") {
  checkSnapshot(targetPath);
} else if (mode === "--text-gates-json") {
  process.stdout.write(renderTextGatePayload());
} else {
  process.stdout.write(renderRegistrySnapshot());
}

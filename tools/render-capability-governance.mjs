import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { CAPABILITY_GOVERNANCE } from "../frontend/src/capabilities/capabilityGovernance.js";

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

function renderRegistrySnapshot() {
  const registry = CAPABILITY_GOVERNANCE.registry;
  const classSummary = summarizeBy(registry, (entry) => entry.class);
  const familySummary = summarizeBy(registry, (entry) => entry.family);
  const familyOrder = [...new Set(registry.map((entry) => entry.family))];

  const lines = [
    "# Generated Capability Governance Registry",
    "",
    "This file is generated from `frontend/src/capabilities/capabilityGovernance.js`.",
    "Do not edit it by hand.",
    "",
    `Schema version: \`${CAPABILITY_GOVERNANCE.schemaVersion}\``,
    "",
    "Regenerate this snapshot with:",
    "",
    "```powershell",
    "powershell -NoProfile -ExecutionPolicy Bypass -File tools\\check-capability-governance.ps1 -WriteSnapshot",
    "```",
    "",
    "## Summary By Class",
    "",
    renderTable(
      ["Class", "Entry Count"],
      classSummary.map(([className, count]) => [className, count])
    ),
    "",
    "## Summary By Family",
    "",
    renderTable(
      ["Family", "Entry Count"],
      familySummary.map(([family, count]) => [family, count])
    )
  ];

  for (const family of familyOrder) {
    const familyEntries = registry.filter((entry) => entry.family === family);
    lines.push(
      "",
      `## ${family}`,
      "",
      renderTable(
        ["ID", "Value", "Class", "Owner Role", "Review Responsibility", "Source Of Truth", "Notes"],
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

function resolveOutputPath(targetPath) {
  if (!targetPath) {
    throw new Error("Missing output path.");
  }
  return path.isAbsolute(targetPath) ? targetPath : path.join(repoRoot, targetPath);
}

function writeSnapshot(targetPath) {
  const resolvedPath = resolveOutputPath(targetPath);
  fs.mkdirSync(path.dirname(resolvedPath), { recursive: true });
  fs.writeFileSync(resolvedPath, renderRegistrySnapshot(), "utf8");
  process.stdout.write(`Capability governance snapshot written to ${resolvedPath}\n`);
}

function checkSnapshot(targetPath) {
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

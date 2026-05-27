import { useEffect, useMemo, useState } from "react";
import { apiClient } from "../api/client";
import { useI18n } from "../i18n";
import { useGraphStore } from "../store/graphStore";
import { humanizeErrorText } from "../utils/errorText";

function shortDigest(value) {
  if (!value) return "-";
  return value.length > 18 ? `${value.slice(0, 18)}...` : value;
}

function safeFileName(value, fallback) {
  const source = typeof value === "string" && value.trim() ? value.trim() : fallback;
  return source.replace(/[^a-zA-Z0-9._-]+/g, "_").replace(/^_+|_+$/g, "") || fallback;
}

function domainLabel(domainId, t) {
  const labels = {
    market: "市场与数据",
    observation: "观察与信号",
    state_machine: "状态机",
    risk: "Risk Plane",
    execution: "执行边界",
    evidence: "证据",
    ai_governance: "AI 治理",
    snapshot: "快照"
  };
  return t(labels[domainId] || domainId);
}

function readinessLabel(value, t) {
  const labels = {
    ready: "已就绪",
    incomplete: "不完整",
    restricted: "受限",
    stale: "需刷新",
    blocked: "已阻断"
  };
  return t(labels[value] || value);
}

function lifecycleLabel(value, t) {
  const labels = {
    implemented: "当前已实现",
    documentable: "当前可文档化",
    milestone: "后续里程碑",
    unsupported: "明确不做"
  };
  return t(labels[value] || value);
}

function decisionLabel(value, t) {
  const labels = {
    ready: "可继续",
    restricted: "受限可继续",
    blocked: "已阻断"
  };
  return t(labels[value] || value);
}

function changeFlagLabel(key, t) {
  const labels = {
    lifecycle_changed: "生命周期",
    readiness_changed: "就绪状态",
    source_refs_changed: "来源证据",
    findings_changed: "诊断"
  };
  return t(labels[key] || key);
}

function primaryActionLabel(value, t) {
  const labels = {
    compile: "编译",
    start_v4_simulation: "启动 v4 模拟",
    preflight: "运行前核验",
    run_backtest: "运行回测",
    review_proposals: "审查 AI 提案",
    create_snapshot: "创建快照"
  };
  return t(labels[value] || value || "无需动作");
}

function sourceKindLabel(value, t) {
  const labels = {
    graph: "策略图",
    qs: "QuantScript",
    core_ir: "Core IR",
    v4_graph: "v4 graph",
    capability: "能力快照",
    runtime_boundary: "运行边界",
    runtime_config: "运行配置",
    capability_snapshot: "能力快照",
    evidence_anchor: "证据锚点",
    proposal_binding: "AI 提案绑定",
    snapshot: "快照"
  };
  return t(labels[value] || value);
}

function findingSeverityLabel(value, t) {
  const labels = {
    info: "提示",
    warning: "警告",
    error: "错误"
  };
  return t(labels[value] || value);
}

function readinessTone(value) {
  if (value === "ready") return "success";
  if (value === "restricted" || value === "stale") return "warning";
  if (value === "blocked") return "danger";
  return "muted";
}

function lifecycleTone(value) {
  if (value === "implemented") return "success";
  if (value === "documentable") return "info";
  if (value === "milestone") return "warning";
  if (value === "unsupported") return "danger";
  return "muted";
}

function severityTone(value) {
  if (value === "error") return "danger";
  if (value === "warning") return "warning";
  return "info";
}

function anchorTypeLabel(anchorType, t) {
  const labels = {
    compile: "编译",
    run: "运行",
    backtest: "回测",
    proposal: "AI 提案",
    snapshot: "快照"
  };
  return t(labels[anchorType] || anchorType);
}

function proposalStatusLabel(value, t) {
  const labels = {
    pending: "待处理",
    passed: "已通过",
    failed: "未通过",
    approved: "已批准",
    rejected: "已拒绝",
    denied: "已拒绝",
    submitted: "已提交",
    static_check_passed: "静态检查通过",
    static_check_failed: "静态检查失败",
    expired: "已过期"
  };
  const resolved = value || "pending";
  return t(labels[resolved] || resolved);
}

function compactList(values) {
  return Array.isArray(values)
    ? values.filter((value) => typeof value === "string" && value.trim()).map((value) => value.trim())
    : [];
}

function collectProposalCandidates(runtime = {}, graph = {}) {
  const metadata = graph?.metadata || {};
  return [
    runtime.aiProposalState?.proposals,
    runtime.aiProposals,
    runtime.ai_proposals,
    runtime.proposals,
    metadata.ai_proposals,
    metadata.proposal_bindings
  ]
    .filter(Array.isArray)
    .flat();
}

function buildProposalBindings(runtime = {}, graph = {}) {
  const seen = new Set();
  return collectProposalCandidates(runtime, graph)
    .map((proposal) => {
      const binding = proposal.config_domain_binding || proposal;
      const proposalId = proposal.proposal_id || proposal.ai_proposal_id || binding.proposal_id;
      const targetDomain = binding.target_domain || proposal.target_domain;
      if (!proposalId || !targetDomain || targetDomain === "unknown") return null;
      const dedupeKey = `${proposalId}:${targetDomain}`;
      if (seen.has(dedupeKey)) return null;
      seen.add(dedupeKey);
      return {
        proposal_id: proposalId,
        target_domain: targetDomain,
        before_digest: binding.before_digest || proposal.before_digest || undefined,
        after_digest: binding.after_digest || proposal.after_digest || undefined,
        evidence_anchor_ids: compactList(binding.evidence_anchor_ids || proposal.evidence_anchor_ids),
        sandbox_status: proposal.sandbox_status || proposal.sandbox_report_status || undefined,
        approval_status: proposal.approval_status || proposal.status || undefined
      };
    })
    .filter(Boolean);
}

function buildEvidenceAnchors(runtime = {}, compileSummary = {}, proposalBindings = []) {
  const anchors = [];
  if (compileSummary.config_hash) {
    anchors.push({
      anchor_type: "compile",
      anchor_id: compileSummary.last_compile_id || "compile",
      digest: compileSummary.config_hash,
      summary: "runtime compile"
    });
  }
  if (runtime.runId) {
    anchors.push({
      anchor_type: "run",
      anchor_id: runtime.runId,
      summary: runtime.runKind || "runtime run"
    });
  }
  if (runtime.selectedBacktestId) {
    anchors.push({
      anchor_type: "backtest",
      anchor_id: runtime.selectedBacktestId,
      summary: "selected backtest"
    });
  }
  proposalBindings.forEach((binding) => {
    anchors.push({
      anchor_type: "proposal",
      anchor_id: binding.proposal_id,
      digest: binding.after_digest || binding.before_digest,
      summary: binding.target_domain
    });
  });
  return anchors;
}

function buildPreflightRequest({ graph, runtime, compileSummary, compileResult, quantScriptDraft, formalQuantScriptDraft, capabilities }) {
  const artifacts = graph?.metadata?.artifacts || {};
  const qsSource =
    formalQuantScriptDraft ||
    artifacts.quantscript?.formal_source ||
    quantScriptDraft ||
    artifacts.quantscript?.graph_source ||
    "";
  const proposalBindings = buildProposalBindings(runtime, graph);

  return {
    strategy_id: graph?.metadata?.graph_id || "local-strategy",
    strategy_version: graph?.metadata?.version || graph?.metadata?.version_label || "local-draft",
    source_mode: graph?.metadata?.source_mode || "strategy_graph",
    graph_json: graph || null,
    runtime_config: compileResult?.runtime_config || graph?.runtime_config || null,
    qs_source: qsSource || null,
    core_ir:
      compileResult?.backend_compile?.core_ir ||
      compileResult?.core_ir ||
      artifacts.core_ir ||
      null,
    capability_snapshot_hash: capabilities?.schema_hash || null,
    capability_source: capabilities?.schema_hash === "safe-fallback" ? "safe_fallback" : "frontend_snapshot",
    runtime_mode: "PaperSimulated",
    evidence_anchors: buildEvidenceAnchors(runtime, compileSummary, proposalBindings),
    proposal_bindings: proposalBindings,
    required_execution_capability_sources: ["runtime_simulated"]
  };
}

export default function StrategyConfigCockpit({ graph, runtime, compileSummary }) {
  const { t } = useI18n();
  const compileResult = useGraphStore((s) => s.compileResult);
  const quantScriptDraft = useGraphStore((s) => s.quantScriptDraft);
  const formalQuantScriptDraft = useGraphStore((s) => s.formalQuantScriptDraft);
  const capabilities = useGraphStore((s) => s.capabilities);
  const [state, setState] = useState({ loading: true, report: null, error: "" });
  const [diffState, setDiffState] = useState({ loading: false, report: null, error: "" });

  const request = useMemo(
    () =>
      buildPreflightRequest({
        graph,
        runtime,
        compileSummary,
        compileResult,
        quantScriptDraft,
        formalQuantScriptDraft,
        capabilities
      }),
    [capabilities, compileResult, compileSummary, formalQuantScriptDraft, graph, quantScriptDraft, runtime]
  );

  const requestKey = useMemo(
    () =>
      JSON.stringify({
        strategy_id: request.strategy_id,
        strategy_version: request.strategy_version,
        graph_updated_at: graph?.metadata?.updated_at,
        compile_hash: compileSummary?.config_hash,
        run_id: runtime?.runId,
        backtest_id: runtime?.selectedBacktestId,
        proposal_bindings: request.proposal_bindings.map((binding) =>
          `${binding.proposal_id}:${binding.target_domain}:${binding.before_digest || "-"}:${binding.after_digest || "-"}:${binding.approval_status || "-"}`
        ),
        capability_snapshot_hash: request.capability_snapshot_hash,
        qs_len: request.qs_source?.length || 0
      }),
    [compileSummary?.config_hash, graph?.metadata?.updated_at, request, runtime?.runId, runtime?.selectedBacktestId]
  );

  useEffect(() => {
    let mounted = true;
    setState((previous) => ({ ...previous, loading: true, error: "" }));
    setDiffState({ loading: false, report: null, error: "" });
    apiClient
      .post("/v1/strategy-config/preflight", request)
      .then((report) => {
        if (!mounted) return;
        setState({ loading: false, report, error: "" });
        const storageKey = `quantpilot.strategy_config.last_artifact.${request.strategy_id}`;
        const previous = readPreviousArtifact(storageKey);
        if (previous?.artifact_digest && report?.artifact?.artifact_digest) {
          if (previous.artifact_digest !== report.artifact.artifact_digest) {
            setDiffState({ loading: true, report: null, error: "" });
            apiClient
              .post("/v1/strategy-config/diff", {
                left: previous,
                right: report.artifact
              })
              .then((diffReport) => {
                if (!mounted) return;
                setDiffState({ loading: false, report: diffReport, error: "" });
              })
              .catch((error) => {
                if (!mounted) return;
                setDiffState({
                  loading: false,
                  report: null,
                  error: humanizeErrorText(error, "策略配置差异核验失败")
                });
              });
          }
        }
        writePreviousArtifact(storageKey, report?.artifact);
      })
      .catch((error) => {
        if (!mounted) return;
        setState({
          loading: false,
          report: null,
          error: humanizeErrorText(error, "策略配置核验失败")
        });
      });
    return () => {
      mounted = false;
    };
  }, [requestKey]);

  const report = state.report;
  const artifact = report?.artifact;
  const domains = artifact?.config_domains || [];
  const boundary = artifact?.runtime_boundary;
  const findings = report?.findings || [];
  const diffReport = diffState.report;
  const [activeDomainId, setActiveDomainId] = useState(null);
  const activeDomain = domains.find((domain) => domain.domain_id === activeDomainId) || domains[0] || null;

  return (
    <div className="dashboard-card" data-testid="strategy-config-cockpit">
      <div className="dashboard-card-header">{t("v4 策略配置")}</div>
      <div className="dashboard-card-body">
        {state.loading && <div className="muted-line">{t("正在核验策略配置...")}</div>}
        {state.error && <div className="muted-line">{state.error}</div>}
        {!state.loading && !state.error && report && (
          <>
            <div className="dashboard-metric">
              <span className="dashboard-metric-label">{t("核验结论")}</span>
              <span className={`dashboard-metric-value ${report.decision === "blocked" ? "danger" : report.decision === "restricted" ? "warning" : "success"}`}>
                {decisionLabel(report.decision, t)}
              </span>
            </div>
            <div className="dashboard-metric">
              <span className="dashboard-metric-label">{t("配置摘要")}</span>
              <span className="dashboard-metric-value">{shortDigest(artifact?.artifact_digest)}</span>
            </div>
            <div className="dashboard-metric">
              <span className="dashboard-metric-label">{t("运行边界")}</span>
              <span className="dashboard-metric-value">
                {boundary?.mode_label || "PaperSimulated"}
                {boundary?.mode_label === "PaperActual" ? ` / ${t("OKX demo")}` : ""}
              </span>
            </div>
            <div className="dashboard-metric">
              <span className="dashboard-metric-label">{t("真实资金")}</span>
              <span className="dashboard-metric-value danger">
                {t("未开放")}
              </span>
            </div>
            <div className="strategy-config-actions">
              <button
                type="button"
                className="ad-btn ad-btn--ghost compact-btn"
                onClick={() => exportStrategyConfigArtifact(artifact)}
                disabled={!artifact}
                data-testid="strategy-config-export-artifact"
              >
                {t("导出配置契约")}
              </button>
            </div>
            <div className="dashboard-domain-grid" data-testid="strategy-config-domains">
              {domains.map((domain) => (
                <div className="dashboard-domain-row" key={domain.domain_id}>
                  <span>{domainLabel(domain.domain_id, t)}</span>
                  <strong>{readinessLabel(domain.readiness, t)}</strong>
                </div>
              ))}
            </div>
            <ConfigDomainRail
              domains={domains}
              activeDomainId={activeDomain?.domain_id}
              onSelect={setActiveDomainId}
              t={t}
            />
            <ConfigDomainPanel
              domain={activeDomain}
              t={t}
            />
            {findings.length > 0 && (
              <div className="dashboard-findings" data-testid="strategy-config-findings">
                {findings.slice(0, 3).map((finding) => (
                  <div className="muted-line" key={`${finding.code}-${finding.message}`}>
                    {finding.message}
                  </div>
                ))}
              </div>
            )}
            <EvidenceAnchorList
              anchors={artifact?.evidence_anchors || []}
              t={t}
            />
            <ProposalBindingPanel
              bindings={artifact?.proposal_bindings || []}
              t={t}
            />
            <StrategyConfigDiffPanel
              diffState={diffState}
              diffReport={diffReport}
              t={t}
            />
          </>
        )}
      </div>
    </div>
  );
}

function readPreviousArtifact(storageKey) {
  if (typeof window === "undefined" || !window.localStorage) return null;
  try {
    const raw = window.localStorage.getItem(storageKey);
    return raw ? JSON.parse(raw) : null;
  } catch (_) {
    return null;
  }
}

function writePreviousArtifact(storageKey, artifact) {
  if (!artifact || typeof window === "undefined" || !window.localStorage) return;
  try {
    window.localStorage.setItem(storageKey, JSON.stringify(artifact));
  } catch (_) {
    // localStorage can be unavailable in hardened desktop shells; diff stays best-effort.
  }
}

function exportStrategyConfigArtifact(artifact) {
  if (!artifact || typeof document === "undefined" || typeof Blob === "undefined") return;
  const payload = JSON.stringify(artifact, null, 2);
  const blob = new Blob([payload], { type: "application/json;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  const strategyId = safeFileName(artifact.strategy_id, "strategy");
  const artifactId = safeFileName(artifact.artifact_id || artifact.artifact_digest, "strategy_config");
  anchor.href = url;
  anchor.download = `${strategyId}-${artifactId}.json`;
  anchor.click();
  window.setTimeout(() => URL.revokeObjectURL(url), 0);
}

function ConfigDomainRail({ domains, activeDomainId, onSelect, t }) {
  if (domains.length === 0) {
    return <div className="muted-line">{t("暂无配置域。")}</div>;
  }
  return (
    <div className="strategy-config-domain-rail" role="tablist" aria-label={t("配置域")} data-testid="strategy-config-domain-rail">
      {domains.map((domain) => {
        const active = domain.domain_id === activeDomainId;
        return (
          <button
            type="button"
            role="tab"
            aria-selected={active}
            className={`ad-btn ad-btn--ghost compact-btn strategy-config-domain-rail__item${active ? " is-active" : ""}`}
            key={domain.domain_id}
            onClick={() => onSelect(domain.domain_id)}
            data-testid={`strategy-config-domain-${domain.domain_id}`}
          >
            <span>{domainLabel(domain.domain_id, t)}</span>
            <span className={`status-pill ${readinessTone(domain.readiness)}`}>
              {readinessLabel(domain.readiness, t)}
            </span>
          </button>
        );
      })}
    </div>
  );
}

function ConfigDomainPanel({ domain, t }) {
  if (!domain) {
    return <div className="muted-line">{t("暂无配置域详情。")}</div>;
  }
  const sourceRefs = domain.source_refs || [];
  const findings = domain.findings || [];
  return (
    <div className="strategy-config-domain-panel" data-testid="strategy-config-domain-panel">
      <div className="strategy-config-domain-panel__header">
        <strong>{domainLabel(domain.domain_id, t)}</strong>
        <span className={`status-pill ${readinessTone(domain.readiness)}`}>
          {readinessLabel(domain.readiness, t)}
        </span>
        <span className={`status-pill ${lifecycleTone(domain.lifecycle)}`}>
          {lifecycleLabel(domain.lifecycle, t)}
        </span>
      </div>
      <div className="dashboard-domain-grid">
        <div className="dashboard-domain-row">
          <span>{t("生命周期")}</span>
          <strong>{lifecycleLabel(domain.lifecycle, t)}</strong>
        </div>
        <div className="dashboard-domain-row">
          <span>{t("就绪状态")}</span>
          <strong>{readinessLabel(domain.readiness, t)}</strong>
        </div>
        <div className="dashboard-domain-row">
          <span>{t("主动作")}</span>
          <strong>{primaryActionLabel(domain.primary_action, t)}</strong>
        </div>
      </div>
      <div className="dashboard-findings" data-testid="strategy-config-domain-sources">
        <div className="muted-line">{t("来源")}</div>
        {sourceRefs.length === 0 ? (
          <div className="muted-line">{t("暂无来源引用。")}</div>
        ) : (
          sourceRefs.slice(0, 5).map((source) => (
            <div className="dashboard-domain-row" key={`${source.source_kind}-${source.source_id}`}>
              <span>
                {sourceKindLabel(source.source_kind, t)} · {source.source_id}
              </span>
              <strong>{shortDigest(source.digest)}</strong>
            </div>
          ))
        )}
      </div>
      <div className="dashboard-findings" data-testid="strategy-config-domain-findings">
        <div className="muted-line">{t("诊断")}</div>
        {findings.length === 0 ? (
          <div className="muted-line">{t("暂无域级诊断。")}</div>
        ) : (
          findings.slice(0, 5).map((finding) => (
            <div className="dashboard-domain-row" key={`${finding.code}-${finding.message}`}>
              <span>
                {findingSeverityLabel(finding.severity, t)} · {finding.code}
              </span>
              <strong>{finding.message}</strong>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

function EvidenceAnchorList({ anchors, t }) {
  return (
    <div className="dashboard-findings" data-testid="strategy-config-evidence-anchors">
      <div className="muted-line">{t("证据锚点")}</div>
      {anchors.length === 0 ? (
        <div className="muted-line">{t("暂无证据锚点。")}</div>
      ) : (
        anchors.slice(0, 5).map((anchor) => (
          <div className="dashboard-domain-row" key={`${anchor.anchor_type}-${anchor.anchor_id}`}>
            <span>
              {anchorTypeLabel(anchor.anchor_type, t)} · {anchor.anchor_id}
            </span>
            <strong>{shortDigest(anchor.digest || anchor.summary)}</strong>
          </div>
        ))
      )}
    </div>
  );
}

function ProposalBindingPanel({ bindings, t }) {
  return (
    <div className="dashboard-findings" data-testid="strategy-config-proposal-bindings">
      <div className="muted-line">{t("AI 提案绑定")}</div>
      {bindings.length === 0 ? (
        <div className="muted-line">{t("暂无 AI 提案绑定。")}</div>
      ) : (
        bindings.slice(0, 5).map((binding) => (
          <div className="dashboard-domain-row" key={`${binding.proposal_id}-${binding.target_domain}`}>
            <span>
              {domainLabel(binding.target_domain, t)} · {binding.proposal_id}
            </span>
            <strong>
              {proposalStatusLabel(binding.sandbox_status, t)} / {proposalStatusLabel(binding.approval_status, t)}
            </strong>
          </div>
        ))
      )}
      {bindings.length > 0 && (
        <div className="muted-line">
          {t("参数摘要")}：{bindings.map((binding) => `${shortDigest(binding.before_digest)} → ${shortDigest(binding.after_digest)}`).join(" / ")}
        </div>
      )}
    </div>
  );
}

function StrategyConfigDiffPanel({ diffState, diffReport, t }) {
  if (diffState.loading) {
    return <div className="muted-line">{t("正在对比上次策略配置...")}</div>;
  }
  if (diffState.error) {
    return <div className="muted-line">{diffState.error}</div>;
  }
  if (!diffReport) {
    return <div className="muted-line">{t("暂无上一份策略配置可对比。")}</div>;
  }
  if (!diffReport.changed) {
    return <div className="muted-line">{t("与上次策略配置一致。")}</div>;
  }
  return (
    <div className="dashboard-findings" data-testid="strategy-config-diff">
      <div className="muted-line">
        {t("配置差异")} · {shortDigest(diffReport.left_artifact_id)} → {shortDigest(diffReport.right_artifact_id)}
      </div>
      {diffReport.runtime_boundary_changed && (
        <div className="muted-line">{t("运行边界发生变化，启动前需要重新核验。")}</div>
      )}
      {(diffReport.domain_changes || []).slice(0, 4).map((change) => {
        const flags = [
          "lifecycle_changed",
          "readiness_changed",
          "source_refs_changed",
          "findings_changed"
        ].filter((key) => change[key]);
        return (
          <div className="dashboard-domain-row" key={change.domain_id}>
            <span>{domainLabel(change.domain_id, t)}</span>
            <strong>{flags.map((key) => changeFlagLabel(key, t)).join(" / ")}</strong>
          </div>
        );
      })}
      {(diffReport.source_digest_changes || []).length > 0 && (
        <div className="muted-line">
          {t("来源摘要变化")}：{diffReport.source_digest_changes.map((change) => change.field).join(", ")}
        </div>
      )}
    </div>
  );
}

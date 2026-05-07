import { useI18n } from "../i18n";
import { useEffect, useState, useCallback } from "react";
import { runtimeStatusLabel } from "../utils/runtimeStatus";
import { useWorkspaceActionBarModel } from "../hooks/useWorkspaceActionBarModel";
import { triggerTutorial } from "../hooks/useTutorial";
import { OkxCredentialInput } from "./CredentialInput";
import { API_BASE } from "../utils/api";

function ToolbarNotices({ capabilityAlert, notice, setNotice }) {
  return (
    <>
      {capabilityAlert ? (
        <div
          className={`toolbar-notice panel-feedback panel-feedback-${capabilityAlert.type} toolbar-notice-${capabilityAlert.type}`}
          role="status"
          data-testid="toolbar-capability-alert"
        >
          <span>{capabilityAlert.message}</span>
        </div>
      ) : null}

      {notice ? (
        <div
          className={`toolbar-notice panel-feedback panel-feedback-${notice.type} toolbar-notice-${notice.type}`}
          role="status"
          data-testid="toolbar-notice"
        >
          <span>{notice.message}</span>
          <button type="button" className="toolbar-notice-close" onClick={() => setNotice(null)} data-testid="toolbar-notice-close">
            关闭
          </button>
        </div>
      ) : null}
    </>
  );
}

function ToolbarIssues({
  graph,
  validationFindings,
  visibleFindings,
  hiddenFindingCount,
  focusFinding,
  className = ""
}) {
  if (validationFindings.length === 0) return null;

  return (
    <div className={`toolbar-issues ${className}`.trim()} aria-label="策略图校验问题">
      <div className="toolbar-issues-title">
        {graph.validation_state.issue_counts?.error > 0 ? "阻塞问题" : "建议修复"}
      </div>
      <div className="toolbar-issues-list">
        {visibleFindings.map((issue) => (
          <button
            key={issue.id}
            type="button"
            className={`toolbar-issue-chip issue-${issue.level}`}
            onClick={() => focusFinding(issue)}
            title={issue.hint ? `${issue.message} ${issue.hint}` : issue.message}
          >
            <span className="toolbar-issue-scope">{issue.scopeLabel}</span>
            <span className="toolbar-issue-message">{issue.message}</span>
          </button>
        ))}
        {hiddenFindingCount > 0 ? (
          <div className="toolbar-issue-overflow">更多 {hiddenFindingCount} 项</div>
        ) : null}
      </div>
    </div>
  );
}

function DefaultToolbarLayout({
  graph,
  runtime,
  statusLabel,
  capabilityLabel,
  runtimeMeta,
  capabilityAlert,
  notice,
  visibleFindings,
  hiddenFindingCount,
  validationFindings,
  formalCompileSourceMeta,
  compileButtonTitle,
  startSimulationTitle,
  runBacktestTitle,
  exportConfigTitle,
  canCompile,
  canStartRuntime,
  canStartBacktest,
  canStopRuntime,
  setNotice,
  handleLoadLatestGraph,
  handleSaveGraph,
  handleExportRuntimeConfig,
  handleExportQuantScript,
  handleCompile,
  handleStartRuntime,
  handleStartBacktest,
  stopRuntime,
  resetRuntime,
  resetGraph,
  focusFinding,
  capabilitySyncBlocked,
  capabilityMessage,
  saving,
  onOpenCredentials
}) {
  return (
    <>
      <div className="top-toolbar-main">
        <div className="toolbar-group">
          <div className="toolbar-brand">
            <div className="toolbar-brand-mark">QP</div>
            <div className="toolbar-brand-copy">
              <div className="toolbar-brand-title">QuantPilot</div>
              <div className="toolbar-brand-subtitle">模拟运行沙盒</div>
            </div>
          </div>
          <button
            className="ghost-btn tutorial-entry-btn"
            onClick={() => triggerTutorial()}
            data-testid="toolbar-tutorial-action"
            title="查看使用教程"
          >
            教程
          </button>
          <button
            className="ghost-btn"
            onClick={onOpenCredentials}
            data-testid="toolbar-credentials-action"
            title="管理交易所凭证"
          >
            凭证
          </button>
          <button className="ghost-btn" onClick={resetGraph} data-testid="toolbar-reset-graph-action">
            新建策略图
          </button>
          <button className="ghost-btn" onClick={handleLoadLatestGraph} data-testid="toolbar-load-latest-action">
            加载最新
          </button>
          <button className="ghost-btn" onClick={handleSaveGraph} disabled={saving} data-testid="toolbar-save-graph-action">
            保存策略图
          </button>
          <button
            className="ghost-btn"
            data-testid="toolbar-export-runtime-config-action"
            onClick={() => handleExportRuntimeConfig({ capabilitySyncBlocked, capabilityMessage })}
            disabled={capabilitySyncBlocked || saving}
            title={exportConfigTitle}
          >
            导出运行配置
          </button>
          <button
            className="ghost-btn"
            data-testid="toolbar-export-quantscript-action"
            onClick={() => handleExportQuantScript({ graph })}
            disabled={saving}
          >
            导出策略图源码
          </button>
        </div>

        <div className="toolbar-center">
          <div className="toolbar-graph-meta">
            <div className="graph-kicker">{graph.metadata.graph_id || "draft_graph"}</div>
            <div className="graph-title">{graph.metadata.name}</div>
          </div>
          <div className={`status-pill ${statusLabel.tone}`}>{statusLabel.text}</div>
          <div className={`status-pill ${capabilityLabel.tone}`}>{capabilityLabel.text}</div>
          <div
            className={`status-pill ${formalCompileSourceMeta.tone}`}
            title={formalCompileSourceMeta.title}
            data-testid="toolbar-formal-source-pill"
          >
            {formalCompileSourceMeta.text}
          </div>
          <div className={`runtime-pill ${runtimeMeta.tone}`}>
            运行时：{runtimeStatusLabel(runtime.status)}
          </div>
        </div>

        <div className="toolbar-group">
          <button
            className="ghost-btn"
            data-testid="toolbar-compile-action"
            onClick={() => handleCompile({ capabilitySyncBlocked, capabilityMessage })}
            disabled={!canCompile}
            title={compileButtonTitle}
          >
            编译
          </button>
          <button
            className="primary-btn"
            data-testid="toolbar-start-runtime-action"
            onClick={() => handleStartRuntime({ graph, capabilitySyncBlocked, capabilityMessage })}
            disabled={!canStartRuntime}
            title={startSimulationTitle}
          >
            启动模拟
          </button>
          <button
            className="ghost-btn"
            data-testid="toolbar-start-backtest-action"
            onClick={() => handleStartBacktest({ graph, capabilitySyncBlocked, capabilityMessage })}
            disabled={!canStartBacktest}
            title={runBacktestTitle}
          >
            运行回测
          </button>
          <button className="ghost-btn" onClick={stopRuntime} disabled={!canStopRuntime} data-testid="toolbar-stop-runtime-action">
            停止
          </button>
          <button className="ghost-btn" onClick={resetRuntime} data-testid="toolbar-reset-runtime-action">
            重置运行时
          </button>
        </div>
      </div>

      <ToolbarNotices
        capabilityAlert={capabilityAlert}
        notice={notice}
        setNotice={setNotice}
      />

      <ToolbarIssues
        graph={graph}
        validationFindings={validationFindings}
        visibleFindings={visibleFindings}
        hiddenFindingCount={hiddenFindingCount}
        focusFinding={focusFinding}
      />
    </>
  );
}

function WorkspaceToolbarLayout({
  graph,
  runtime,
  statusLabel,
  capabilityLabel,
  runtimeMeta,
  capabilityAlert,
  notice,
  visibleFindings,
  hiddenFindingCount,
  validationFindings,
  formalCompileSourceMeta,
  compileButtonTitle,
  startSimulationTitle,
  runBacktestTitle,
  exportConfigTitle,
  canCompile,
  canStartRuntime,
  canStartBacktest,
  canStopRuntime,
  setNotice,
  handleLoadLatestGraph,
  handleSaveGraph,
  handleExportRuntimeConfig,
  handleExportQuantScript,
  handleCompile,
  handleStartRuntime,
  handleStartBacktest,
  stopRuntime,
  resetRuntime,
  resetGraph,
  focusFinding,
  capabilitySyncBlocked,
  capabilityMessage,
  saving,
  onOpenCredentials
}) {
  return (
    <>
      <div className="top-toolbar-main top-toolbar-main--workspace">
        <div className="toolbar-center toolbar-center--workspace">
          <div className="toolbar-workspace-context">
            <div className="toolbar-workspace-context__label">工作区操作</div>
            <div className="toolbar-graph-meta">
              <div className="graph-kicker">{graph.metadata.graph_id || "draft_graph"}</div>
              <div className="graph-title">{graph.metadata.name}</div>
            </div>
          </div>
          <div className="toolbar-status-strip">
            <div className={`status-pill ${statusLabel.tone}`}>{statusLabel.text}</div>
            <div className={`status-pill ${capabilityLabel.tone}`}>{capabilityLabel.text}</div>
            <div
              className={`status-pill ${formalCompileSourceMeta.tone}`}
              title={formalCompileSourceMeta.title}
              data-testid="toolbar-formal-source-pill"
            >
              {formalCompileSourceMeta.text}
            </div>
            <div className={`runtime-pill ${runtimeMeta.tone}`}>
              运行时：{runtimeStatusLabel(runtime.status)}
            </div>
          </div>
        </div>

        <div className="toolbar-group toolbar-group--workspace-primary">
          <button
            className="ghost-btn tutorial-entry-btn"
            onClick={() => triggerTutorial()}
            data-testid="toolbar-tutorial-action"
            title="查看使用教程"
          >
            教程
          </button>
          <button
            className="ghost-btn"
            data-testid="toolbar-compile-action"
            onClick={() => handleCompile({ capabilitySyncBlocked, capabilityMessage })}
            disabled={!canCompile}
            title={compileButtonTitle}
          >
            编译
          </button>
          <button
            className="primary-btn"
            data-testid="toolbar-start-runtime-action"
            onClick={() => handleStartRuntime({ graph, capabilitySyncBlocked, capabilityMessage })}
            disabled={!canStartRuntime}
            title={startSimulationTitle}
          >
            启动模拟
          </button>
          <button
            className="ghost-btn"
            data-testid="toolbar-start-backtest-action"
            onClick={() => handleStartBacktest({ graph, capabilitySyncBlocked, capabilityMessage })}
            disabled={!canStartBacktest}
            title={runBacktestTitle}
          >
            运行回测
          </button>
          <button className="ghost-btn" onClick={stopRuntime} disabled={!canStopRuntime} data-testid="toolbar-stop-runtime-action">
            停止
          </button>
          <button className="ghost-btn" onClick={resetRuntime} data-testid="toolbar-reset-runtime-action">
            重置运行时
          </button>
        </div>
      </div>

      <div className="top-toolbar-utility-row">
        <div className="top-toolbar-utility-row__label">工具</div>
        <div className="toolbar-group toolbar-group--workspace-secondary">
          <button className="ghost-btn" onClick={handleSaveGraph} disabled={saving} data-testid="toolbar-save-graph-action">
            保存策略图
          </button>
          <button className="ghost-btn" onClick={handleLoadLatestGraph} data-testid="toolbar-load-latest-action">
            加载最新
          </button>
          <button
            className="ghost-btn"
            data-testid="toolbar-export-runtime-config-action"
            onClick={() => handleExportRuntimeConfig({ capabilitySyncBlocked, capabilityMessage })}
            disabled={capabilitySyncBlocked || saving}
            title={exportConfigTitle}
          >
            导出运行配置
          </button>
          <button
            className="ghost-btn"
            data-testid="toolbar-export-quantscript-action"
            onClick={() => handleExportQuantScript({ graph })}
            disabled={saving}
          >
            导出策略图源码
          </button>
          <button className="ghost-btn" onClick={resetGraph} data-testid="toolbar-reset-graph-action">
            新建策略图
          </button>
        </div>
      </div>

      <ToolbarNotices
        capabilityAlert={capabilityAlert}
        notice={notice}
        setNotice={setNotice}
      />

      <ToolbarIssues
        graph={graph}
        validationFindings={validationFindings}
        visibleFindings={visibleFindings}
        hiddenFindingCount={hiddenFindingCount}
        focusFinding={focusFinding}
        className="toolbar-issues--workspace"
      />
    </>
  );
}

function CredentialPanel({ onClose }) {
  const { t } = useI18n();
  const [services, setServices] = useState([]);
  const [selected, setSelected] = useState(null);
  const [loaded, setLoaded] = useState(false);

  const loadServices = useCallback(async () => {
    try {
      const res = await fetch(API_BASE + "/api/credentials");
      if (res.ok) {
        const data = await res.json();
        setServices(data.services || []);
      }
    } catch (_) { /* 凭证 API 可能未就绪 */ }
    setLoaded(true);
  }, []);

  useEffect(() => { loadServices(); }, [loadServices]);

  const handleSave = useCallback(async (label, fields) => {
    try {
      const res = await fetch(API_BASE + "/api/credentials", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ service: label, fields }),
      });
      if (res.ok) {
        setSelected(null);
        loadServices();
      }
    } catch (_) {}
  }, [loadServices]);

  const handleDelete = useCallback(async (label) => {
    try {
      await fetch(API_BASE + "/api/credentials/" + encodeURIComponent(label), { method: "DELETE" });
      loadServices();
    } catch (_) {}
  }, [loadServices]);

  if (selected === "new" || (selected && !services.includes(selected))) {
    return (
      <div className="credential-panel" data-testid="credential-panel">
        <div className="credential-panel-header">
          <span>{selected === "new" ? t("新增凭证") : t("编辑凭证")}</span>
          <button className="ghost-btn compact-btn" onClick={() => setSelected(null)}>{t("返回")}</button>
        </div>
        <OkxCredentialInput
          label={selected === "new" ? "" : selected}
          onSave={handleSave}
          onCancel={() => setSelected(null)}
        />
      </div>
    );
  }

  return (
    <div className="credential-panel" data-testid="credential-panel">
      <div className="credential-panel-header">
        <span>{t("凭证管理")}</span>
        <button className="ghost-btn compact-btn" onClick={onClose}>{t("关闭")}</button>
      </div>
      {!loaded ? (
        <div className="credential-panel-empty">{t("加载中...")}</div>
      ) : services.length === 0 ? (
        <div className="credential-panel-empty">{t("暂无已存储凭证")}</div>
      ) : (
        <ul className="credential-list">
          {services.map((s) => (
            <li key={s} className="credential-list-item">
              <span>{s}</span>
              <div>
                <button className="ghost-btn compact-btn" onClick={() => setSelected(s)}>{t("编辑")}</button>
                <button className="ghost-btn compact-btn" onClick={() => handleDelete(s)}>{t("删除")}</button>
              </div>
            </li>
          ))}
        </ul>
      )}
      <button className="primary-btn" style={{ marginTop: 12 }} onClick={() => setSelected("new")}>
        {t("新增凭证")}
      </button>
    </div>
  );
}

export default function TopToolbar({ variant = "default" }) {
  useI18n();
  const model = useWorkspaceActionBarModel();
  const isWorkspace = variant === "workspace";
  const [saving, setSaving] = useState(false);
  const [showCredentials, setShowCredentials] = useState(false);

  const guardedSaveGraph = useCallback(async () => {
    setSaving(true);
    try { await model.handleSaveGraph(); } finally { setSaving(false); }
  }, [model.handleSaveGraph]);

  const guardedExportQuantScript = useCallback(async () => {
    setSaving(true);
    try { await model.handleExportQuantScript({ graph: model.graph }); } finally { setSaving(false); }
  }, [model.handleExportQuantScript, model.graph]);

  const guardedExportRuntimeConfig = useCallback(async (opts) => {
    setSaving(true);
    try { await model.handleExportRuntimeConfig(opts); } finally { setSaving(false); }
  }, [model.handleExportRuntimeConfig]);

  return (
    <header className={`top-toolbar${isWorkspace ? " top-toolbar--workspace" : ""}`}>
      {isWorkspace ? (
        <WorkspaceToolbarLayout {...model} saving={saving} onOpenCredentials={() => setShowCredentials(true)} handleSaveGraph={guardedSaveGraph} handleExportQuantScript={guardedExportQuantScript} handleExportRuntimeConfig={guardedExportRuntimeConfig} />
      ) : (
        <DefaultToolbarLayout {...model} saving={saving} onOpenCredentials={() => setShowCredentials(true)} handleSaveGraph={guardedSaveGraph} handleExportQuantScript={guardedExportQuantScript} handleExportRuntimeConfig={guardedExportRuntimeConfig} />
      )}
      {showCredentials ? (
        <CredentialPanel onClose={() => setShowCredentials(false)} />
      ) : null}
    </header>
  );
}

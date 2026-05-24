import { useI18n } from "../i18n";
import { useEffect, useState, useCallback } from "react";
import { runtimeStatusLabel } from "../utils/runtimeStatus";
import { useWorkspaceActionBarModel } from "../hooks/useWorkspaceActionBarModel";
import { triggerTutorial } from "../hooks/useTutorial";
import { OkxCredentialInput } from "./CredentialInput";
import DeployButton from "./DeployButton";
import { API_BASE } from "../utils/api";
import { useGraphStore } from "../store/graphStore";

function ToolbarNotices({ capabilityAlert, notice, setNotice }) {
  const { t } = useI18n();
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
            {t("关闭")}
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
  const { t } = useI18n();
  if (validationFindings.length === 0) return null;

  return (
    <div className={`toolbar-issues ${className}`.trim()} aria-label={t("策略图校验问题")}>
      <div className="toolbar-issues-title">
        {graph.validation_state.issue_counts?.error > 0 ? t("阻塞问题") : t("建议修复")}
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
          <div className="toolbar-issue-overflow">{t("更多")} {hiddenFindingCount} {t("项")}</div>
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
  saveGraphTitle,
  loadLatestTitle,
  resetGraphTitle,
  exportQuantScriptTitle,
  stopRuntimeTitle,
  resetRuntimeTitle,
  tutorialTitle,
  credentialsTitle,
  issueSummary,
  canCompile,
  canStartRuntime,
  canStartBacktest,
  canStopRuntime,
  canSaveGraph,
  canLoadLatestGraph,
  canResetGraph,
  canResetRuntime,
  canExportRuntimeConfig,
  canExportQuantScript,
  canOpenTutorial,
  canOpenCredentials,
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
  onOpenCredentials,
  isCompiling
}) {
  const { t } = useI18n();
  const isBusy = runtime.status === "running" || runtime.status === "backtesting";
  return (
    <>
      <div className="top-toolbar-main">
        <div className="toolbar-group">
          <div className="toolbar-brand">
            <div className="toolbar-brand-mark">QP</div>
            <div className="toolbar-brand-copy">
              <div className="toolbar-brand-title">QuantPilot</div>
              <div className="toolbar-brand-subtitle">{t("模拟运行沙盒")}</div>
            </div>
          </div>
          <button
            className="ghost-btn tutorial-entry-btn"
            onClick={() => triggerTutorial()}
            data-testid="toolbar-tutorial-action"
            disabled={!canOpenTutorial}
            title={tutorialTitle}
          >
            {t("教程")}
          </button>
          <button
            className="ghost-btn"
            onClick={onOpenCredentials}
            data-testid="toolbar-credentials-action"
            disabled={!canOpenCredentials}
            title={credentialsTitle}
          >
            {t("凭证")}
          </button>
          <button className="ghost-btn" onClick={() => { if (window.confirm(t("确认新建策略图？当前未保存的更改将丢失。"))) resetGraph(); }} disabled={!canResetGraph} title={resetGraphTitle} data-testid="toolbar-reset-graph-action">
            {t("新建策略图")}
          </button>
          <button className="ghost-btn" onClick={handleLoadLatestGraph} disabled={!canLoadLatestGraph} title={loadLatestTitle} data-testid="toolbar-load-latest-action">
            {t("加载最新")}
          </button>
          <button className="ghost-btn" onClick={handleSaveGraph} disabled={saving || !canSaveGraph} title={saveGraphTitle} data-testid="toolbar-save-graph-action">
            {saving ? t("保存中...") : t("保存策略图")}
          </button>
          <button
            className="ghost-btn"
            data-testid="toolbar-export-runtime-config-action"
            onClick={() => handleExportRuntimeConfig({ capabilitySyncBlocked, capabilityMessage })}
            disabled={!canExportRuntimeConfig || saving}
            title={exportConfigTitle}
          >
            {t("导出运行配置")}
          </button>
          <button
            className="ghost-btn"
            data-testid="toolbar-export-quantscript-action"
            onClick={() => handleExportQuantScript({ graph })}
            disabled={saving || !canExportQuantScript}
            title={exportQuantScriptTitle}
          >
            {t("导出策略图源码")}
          </button>
        </div>

        <div className="toolbar-center">
          <div className="toolbar-graph-meta">
            <div className="graph-kicker">{graph.metadata?.graph_id || "draft_graph"}</div>
            <div className="graph-title">{graph.metadata?.name || t("未命名策略")}</div>
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
            {isBusy ? (runtime.status === "running" ? t("模拟中...") : t("回测中...")) : `运行时：${runtimeStatusLabel(runtime.status)}`}
          </div>
        </div>

        <div className="toolbar-group">
          <button
            className="ghost-btn"
            data-testid="toolbar-compile-action"
            onClick={() => handleCompile({ capabilitySyncBlocked, capabilityMessage })}
            disabled={!canCompile || isBusy || isCompiling}
            title={compileButtonTitle}
          >
            {isCompiling ? t("编译中...") : `编译${issueSummary ? ` (${issueSummary})` : ""}`}
          </button>
          <button
            className="primary-btn"
            data-testid="toolbar-start-runtime-action"
            onClick={() => handleStartRuntime({ graph, capabilitySyncBlocked, capabilityMessage })}
            disabled={!canStartRuntime || isBusy}
            title={startSimulationTitle}
          >
            {runtime.status === "running" ? t("模拟中...") : t("启动模拟")}
          </button>
          <button
            className="ghost-btn"
            data-testid="toolbar-start-backtest-action"
            onClick={() => handleStartBacktest({ graph, capabilitySyncBlocked, capabilityMessage })}
            disabled={!canStartBacktest || isBusy}
            title={runBacktestTitle}
          >
            {runtime.status === "backtesting" ? t("回测中...") : t("运行回测")}
          </button>
          <button className="ghost-btn" onClick={() => { if (window.confirm(t("确认停止当前模拟？"))) stopRuntime(); }} disabled={!canStopRuntime} title={stopRuntimeTitle} data-testid="toolbar-stop-runtime-action">
            {t("停止")}
          </button>
          <button className="ghost-btn" onClick={() => { if (window.confirm(t("确认重置运行时？运行中的模拟将被中断。"))) resetRuntime(); }} disabled={!canResetRuntime} title={resetRuntimeTitle} data-testid="toolbar-reset-runtime-action">
            {t("重置运行时")}
          </button>
          <DeployButton graph={graph} canDeploy={canCompile} />
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
  saveGraphTitle,
  loadLatestTitle,
  resetGraphTitle,
  exportQuantScriptTitle,
  stopRuntimeTitle,
  resetRuntimeTitle,
  tutorialTitle,
  credentialsTitle,
  issueSummary,
  canCompile,
  canStartRuntime,
  canStartBacktest,
  canStopRuntime,
  canSaveGraph,
  canLoadLatestGraph,
  canResetGraph,
  canResetRuntime,
  canExportRuntimeConfig,
  canExportQuantScript,
  canOpenTutorial,
  canOpenCredentials,
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
  onOpenCredentials,
  isCompiling
}) {
  const { t } = useI18n();
  const [toolsOpen, setToolsOpen] = useState(false);
  const toolsPanelId = "workspace-toolbar-tools-panel";
  return (
    <>
      <div className="top-toolbar-main top-toolbar-main--workspace">
        <div className="toolbar-center toolbar-center--workspace">
          <div className="toolbar-workspace-context">
            <div className="toolbar-workspace-context__label">工作区操作</div>
            <div className="toolbar-graph-meta">
              <div className="graph-kicker">{graph.metadata?.graph_id || "draft_graph"}</div>
              <div className="graph-title">{graph.metadata?.name || t("未命名策略")}</div>
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
            className="ghost-btn"
            data-testid="toolbar-compile-action"
            onClick={() => handleCompile({ capabilitySyncBlocked, capabilityMessage })}
            disabled={!canCompile || isCompiling}
            title={compileButtonTitle}
          >
            {isCompiling ? "编译中..." : `编译${issueSummary ? ` (${issueSummary})` : ""}`}
          </button>
          <button
            className="primary-btn"
            data-testid="toolbar-start-runtime-action"
            onClick={() => handleStartRuntime({ graph, capabilitySyncBlocked, capabilityMessage })}
            disabled={!canStartRuntime}
            title={startSimulationTitle}
          >
            {t("启动模拟")}
          </button>
          <button
            className="ghost-btn"
            data-testid="toolbar-start-backtest-action"
            onClick={() => handleStartBacktest({ graph, capabilitySyncBlocked, capabilityMessage })}
            disabled={!canStartBacktest}
            title={runBacktestTitle}
          >
            {t("运行回测")}
          </button>
          <button className="ghost-btn" onClick={() => { if (window.confirm(t("确认停止当前模拟？"))) stopRuntime(); }} disabled={!canStopRuntime} title={stopRuntimeTitle} data-testid="toolbar-stop-runtime-action">
            {t("停止")}
          </button>
          <button
            type="button"
            className="ghost-btn"
            data-testid="toolbar-workspace-tools-toggle"
            aria-expanded={toolsOpen}
            aria-controls={toolsPanelId}
            onClick={() => setToolsOpen((value) => !value)}
          >
            {toolsOpen ? "收起工具" : "工具"}
          </button>
        </div>
      </div>

      {toolsOpen ? (
        <div className="top-toolbar-utility-row" id={toolsPanelId}>
          <div className="top-toolbar-utility-row__label">工具</div>
          <div className="toolbar-group toolbar-group--workspace-secondary">
            <button
              className="ghost-btn tutorial-entry-btn"
              onClick={() => {
                setToolsOpen(false);
                triggerTutorial();
              }}
              data-testid="toolbar-tutorial-action"
              disabled={!canOpenTutorial}
              title={tutorialTitle}
            >
              教程
            </button>
            <button
              className="ghost-btn"
              onClick={() => {
                setToolsOpen(false);
                onOpenCredentials?.();
              }}
              data-testid="toolbar-credentials-action"
              disabled={!canOpenCredentials}
              title={credentialsTitle}
            >
              {t("凭证")}
            </button>
            <button className="ghost-btn" onClick={handleSaveGraph} disabled={saving || !canSaveGraph} title={saveGraphTitle} data-testid="toolbar-save-graph-action">
              {saving ? "保存中..." : "保存策略图"}
            </button>
            <button className="ghost-btn" onClick={handleLoadLatestGraph} disabled={!canLoadLatestGraph} title={loadLatestTitle} data-testid="toolbar-load-latest-action">
              加载最新
            </button>
            <button
              className="ghost-btn"
              data-testid="toolbar-export-runtime-config-action"
              onClick={() => handleExportRuntimeConfig({ capabilitySyncBlocked, capabilityMessage })}
              disabled={!canExportRuntimeConfig || saving}
              title={exportConfigTitle}
            >
              {t("导出运行配置")}
            </button>
            <button
              className="ghost-btn"
              data-testid="toolbar-export-quantscript-action"
              onClick={() => handleExportQuantScript({ graph })}
              disabled={saving || !canExportQuantScript}
              title={exportQuantScriptTitle}
            >
              {t("导出策略图源码")}
            </button>
            <button className="ghost-btn" onClick={() => { if (window.confirm(t("确认新建策略图？当前未保存的更改将丢失。"))) resetGraph(); }} disabled={!canResetGraph} title={resetGraphTitle} data-testid="toolbar-reset-graph-action">
              {t("新建策略图")}
            </button>
            <button className="ghost-btn" onClick={() => { if (window.confirm(t("确认重置运行时？运行中的模拟将被中断。"))) resetRuntime(); }} disabled={!canResetRuntime} title={resetRuntimeTitle} data-testid="toolbar-reset-runtime-action">
              {t("重置运行时")}
            </button>
          </div>
        </div>
      ) : null}

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
  const [saveError, setSaveError] = useState(null);

  const loadServices = useCallback(async () => {
    try {
      const res = await fetch(API_BASE + "/credentials");
      if (res.ok) {
        const data = await res.json();
        setServices(data.services || []);
      }
    } catch (_) { /* 凭证 API 可能未就绪 */ }
    setLoaded(true);
  }, []);

  useEffect(() => { loadServices(); }, [loadServices]);

  // 关闭面板时主动清零 state，防止凭证明文残留在 React 内存中
  const clearAndClose = useCallback(() => {
    setServices([]);
    setSelected(null);
    setLoaded(false);
    onClose?.();
  }, [onClose]);

  const handleSave = useCallback(async (label, fields) => {
    setSaveError(null);
    try {
      const res = await fetch(API_BASE + "/credentials", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ service: label, fields }),
      });
      if (res.ok) {
        setSelected(null);
        loadServices();
      } else {
        const text = await res.text();
        setSaveError(text || "保存凭证失败，请重试。");
      }
    } catch (e) {
      setSaveError(e.message || "网络错误，请检查连接后重试。");
    }
  }, [loadServices]);

  const handleDelete = useCallback(async (label) => {
    if (!window.confirm(t("确认删除凭证\"{label}\"？此操作不可撤销。", { label }))) return;
    try {
      const res = await fetch(API_BASE + "/credentials/" + encodeURIComponent(label), { method: "DELETE" });
      if (!res.ok) {
        setSaveError("删除凭证失败，请重试。");
      } else {
        loadServices();
      }
    } catch (e) {
      setSaveError(e.message || "网络错误，删除凭证失败。");
    }
  }, [loadServices]);

  if (selected === "new" || (selected && !services.includes(selected))) {
    return (
      <div className="credential-panel" data-testid="credential-panel">
        <div className="credential-panel-header">
          <span>{selected === "new" ? t("新增凭证") : t("编辑凭证") + ": " + selected}</span>
          <button className="ghost-btn compact-btn" onClick={() => setSelected(null)}>{t("返回")}</button>
        </div>
        {selected !== "new" ? (
          <div className="credential-panel-empty" style={{ marginBottom: 12 }}>
            {t("编辑模式下所有字段均需重新填写, 留空的字段将被清除。")}
          </div>
        ) : null}
        {saveError && <div className="qp-error" style={{marginBottom:12}}>{saveError}</div>}
        <OkxCredentialInput
          label={selected === "new" ? "" : selected}
          onSave={handleSave}
          onCancel={() => { setSelected(null); setSaveError(null); }}
        />
      </div>
    );
  }

  return (
    <div className="credential-panel" data-testid="credential-panel">
      <div className="credential-panel-header">
        <span>{t("凭证管理")}</span>
        <button className="ghost-btn compact-btn" onClick={clearAndClose}>{t("关闭")}</button>
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
  const model = useWorkspaceActionBarModel();
  const actionLock = useGraphStore((state) => state.actionLock);
  const isCompiling = actionLock === "compiling";
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
        <WorkspaceToolbarLayout {...model} saving={saving} isCompiling={isCompiling} onOpenCredentials={() => setShowCredentials(true)} handleSaveGraph={guardedSaveGraph} handleExportQuantScript={guardedExportQuantScript} handleExportRuntimeConfig={guardedExportRuntimeConfig} />
      ) : (
        <DefaultToolbarLayout {...model} saving={saving} isCompiling={isCompiling} onOpenCredentials={() => setShowCredentials(true)} handleSaveGraph={guardedSaveGraph} handleExportQuantScript={guardedExportQuantScript} handleExportRuntimeConfig={guardedExportRuntimeConfig} />
      )}
      {showCredentials ? (
        <CredentialPanel onClose={() => setShowCredentials(false)} />
      ) : null}
    </header>
  );
}

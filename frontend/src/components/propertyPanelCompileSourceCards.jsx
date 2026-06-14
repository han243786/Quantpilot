import { useEffect } from "react";
import { translateText } from "../i18n";
import {
  booleanStatusText,
  booleanStatusTone,
  compileOutputsText,
  diagnosticSeverityCounts,
  runtimeSourceOfTruthText,
  runtimeSourceText,
  strategyIrRoleText
} from "../hooks/propertyPanelShared";
import { compileConflictGuidance, compileConflictSummary } from "../utils/compileContract";
import { PropertySubsection, StatusChip } from "./propertyPanelLayoutPrimitives";

export function RepairPathContextPanel({
  insight,
  title = "修复路径上下文",
  summary = "修改字段前，先确认当前变更仍与激活的修复路径保持一致。"
}) {
  if (!insight) return null;

  return (
    <div className="workspace-inspector-shell__context-card">
      <div className="workspace-inspector-shell__context-kicker">修复路径</div>
      <div className="workspace-inspector-shell__context-title">{title}</div>
      <div className="workspace-inspector-shell__context-summary">{summary}</div>
      <div className="workspace-inspector-shell__context-chips">
        <span className="diagnostic-chip diagnostic-chip--path">{insight.chip}</span>
        <span className="diagnostic-chip diagnostic-chip--segment">{insight.segment}</span>
      </div>
      {insight.note ? (
        <div className="workspace-inspector-shell__context-note">{insight.note}</div>
      ) : null}
    </div>
  );
}

export function GraphOverviewCard({ graph, compileSummary }) {
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">策略图概览</div>
        <div className="property-card-caption">
          来源模式：{graph.metadata.source_mode || translateText("未设置")}
        </div>
      </div>
      <div className="kv-line">
        <span>名称</span>
        <strong>{graph.metadata.name}</strong>
      </div>
      <div className="kv-line">
        <span>图 ID</span>
        <strong>{graph.metadata.graph_id || "-"}</strong>
      </div>
      <div className="kv-line">
        <span>节点数</span>
        <strong>{graph.nodes.length}</strong>
      </div>
      <div className="kv-line">
        <span>边数</span>
        <strong>{graph.edges.length}</strong>
      </div>
      <div className="kv-line">
        <span>可编译</span>
        <strong>
          <StatusChip tone={booleanStatusTone(compileSummary.compilable)}>
            {booleanStatusText(compileSummary.compilable)}
          </StatusChip>
        </strong>
      </div>
      <div className="kv-line">
        <span>错误</span>
        <strong>{graph.validation_state?.issue_counts?.error || 0}</strong>
      </div>
      <div className="kv-line">
        <span>警告</span>
        <strong>{graph.validation_state?.issue_counts?.warning || 0}</strong>
      </div>
    </div>
  );
}

export function CompileSummaryCard({ compileSummary }) {
  const strategyIrCheck = compileSummary?.strategy_ir_check || null;
  const artifactResolution = compileSummary?.artifact_resolution || null;
  const diagnostics = Array.isArray(compileSummary?.diagnostics) ? compileSummary.diagnostics : [];
  const severityCounts = diagnosticSeverityCounts(diagnostics);
  const runtimeCompileConflict = compileConflictSummary({
    strategyIrCheck,
    compileSummary
  });
  const conflictGuidance = compileConflictGuidance();

  return (
    <div className="property-card property-card-structured" data-testid="compile-summary-card">
      <div className="property-card-heading">
        <div className="property-card-title">编译摘要</div>
        <div className="property-card-caption">
          将语义预检与最终决定可运行输出的编译结论清晰区分。
        </div>
      </div>

      <PropertySubsection title="编译结论">
        <div className="kv-line">
          <span>可编译</span>
          <strong>
            <StatusChip tone={booleanStatusTone(compileSummary.compilable)}>
              {booleanStatusText(compileSummary.compilable)}
            </StatusChip>
          </strong>
        </div>
        <div className="kv-line">
          <span>最近编译 ID</span>
          <strong>{compileSummary.last_compile_id || "-"}</strong>
        </div>
        <div className="kv-line">
          <span>后端已校验</span>
          <strong>
            <StatusChip tone={booleanStatusTone(compileSummary.backend_verified)}>
              {booleanStatusText(compileSummary.backend_verified)}
            </StatusChip>
          </strong>
        </div>
        <div className="kv-line">
          <span>协议</span>
          <strong>{compileSummary.protocol_name || "-"}</strong>
        </div>
        <div className="kv-line">
          <span>配置哈希</span>
          <strong>{compileSummary.config_hash || "-"}</strong>
        </div>
        <div className="kv-line">
          <span>输出计数</span>
          <strong>{compileOutputsText(compileSummary.outputs)}</strong>
        </div>
      </PropertySubsection>

      <PropertySubsection title="编译消息">
        {compileSummary.errors?.length ? (
          compileSummary.errors.map((message, index) => (
            <div key={`compile_error_${index}`} className="issue-row issue-error">
              <div className="issue-msg">{message}</div>
            </div>
          ))
        ) : (
          <div className="muted-line">当前没有编译错误。</div>
        )}
        {compileSummary.warnings?.length
          ? compileSummary.warnings.map((message, index) => (
              <div key={`compile_warning_${index}`} className="issue-row issue-warning">
                <div className="issue-msg">{message}</div>
              </div>
            ))
          : null}
        {runtimeCompileConflict ? (
          <div className="issue-row issue-warning">
            <div className="issue-msg">{conflictGuidance.message}</div>
            <div className="issue-hint">{conflictGuidance.hint}</div>
          </div>
        ) : null}
      </PropertySubsection>

      <PropertySubsection title="策略中间表示预检">
        <div className="kv-line">
          <span>是否执行</span>
          <strong>{strategyIrCheck?.performed ? "是" : "否"}</strong>
        </div>
        <div className="kv-line">
          <span>可编译</span>
          <strong>
            <StatusChip tone={booleanStatusTone(strategyIrCheck?.compilable)}>
              {booleanStatusText(strategyIrCheck?.compilable)}
            </StatusChip>
          </strong>
        </div>
        <div className="kv-line">
          <span>编译 ID</span>
          <strong>{strategyIrCheck?.compile_id || "-"}</strong>
        </div>
        <div className="kv-line">
          <span>已生成核心中间表示</span>
          <strong>
            <StatusChip tone={booleanStatusTone(strategyIrCheck?.has_core_ir)}>
              {booleanStatusText(strategyIrCheck?.has_core_ir)}
            </StatusChip>
          </strong>
        </div>
      </PropertySubsection>

      <PropertySubsection title="运行真源">
        <div className="kv-line">
          <span>策略中间表示角色</span>
          <strong>{strategyIrRoleText(artifactResolution)}</strong>
        </div>
        <div className="kv-line">
          <span>运行时编译输出</span>
          <strong>{runtimeSourceText(artifactResolution)}</strong>
        </div>
        <div className="kv-line">
          <span>最终可运行输出遵循</span>
          <strong>{runtimeSourceOfTruthText(artifactResolution)}</strong>
        </div>
        {artifactResolution?.notes?.map((note, index) => (
          <div key={`artifact_note_${index}`} className="muted-line">
            {note}
          </div>
        ))}
      </PropertySubsection>

      <PropertySubsection title="诊断级别">
        <div className="kv-line">
          <span>阻塞</span>
          <strong>{severityCounts.blocker}</strong>
        </div>
        <div className="kv-line">
          <span>警告</span>
          <strong>{severityCounts.warning}</strong>
        </div>
        <div className="kv-line">
          <span>提示</span>
          <strong>{severityCounts.info}</strong>
        </div>
      </PropertySubsection>
    </div>
  );
}

export function QuantScriptEditorCard({
  graphSource,
  sourceMode,
  applyError,
  updateQuantScriptDraft,
  handleResetQuantScript,
  handleApplyQuantScript,
  setApplyError,
  onActivateSourceLane = null,
  graphSourceEditorRef = null,
  activeSourceSelection = null
}) {
  useEffect(() => {
    if (!graphSourceEditorRef?.current || !activeSourceSelection) return;
    const [start, end] = activeSourceSelection;
    graphSourceEditorRef.current.focus();
    graphSourceEditorRef.current.setSelectionRange(start, end);
  }, [graphSourceEditorRef, activeSourceSelection]);

  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">策略图源码</div>
        <div className="property-card-caption">
          先在这里修改 `strategy_graph` 草稿，再回写到编辑器。
        </div>
      </div>
      <textarea
        ref={graphSourceEditorRef}
        aria-label="策略图源码"
        value={graphSource}
        rows={14}
        onChange={(event) => updateQuantScriptDraft(event.target.value)}
      />
      {applyError ? (
        <div className="issue-row issue-error">
          <div className="issue-msg">{applyError}</div>
        </div>
      ) : null}
      <div className="kv-line">
        <span>来源模式</span>
        <strong>{sourceMode || "未设置"}</strong>
      </div>
      <div className="toolbar-group">
        <button
          className="ad-btn ad-btn--ghost"
          onClick={() => {
            onActivateSourceLane?.();
            handleResetQuantScript(setApplyError);
          }}
        >
          重置策略图源码
        </button>
        <button
          className="ad-btn ad-btn--primary"
          onClick={() => {
            onActivateSourceLane?.();
            handleApplyQuantScript(setApplyError);
          }}
        >
          应用策略图源码
        </button>
      </div>
    </div>
  );
}

export function FormalQuantScriptEditorCard({
  formalQuantScriptSource,
  formalQuantScriptOverrideActive,
  formalApplyError,
  updateFormalQuantScriptDraft,
  handleResetFormalQuantScript,
  handleApplyFormalQuantScript,
  setFormalApplyError,
  onActivateSourceLane = null,
  formalSourceEditorRef = null,
  activeFormalSourceSelection = null
}) {
  useEffect(() => {
    if (!formalSourceEditorRef?.current || !activeFormalSourceSelection) return;
    const [start, end] = activeFormalSourceSelection;
    formalSourceEditorRef.current.focus();
    formalSourceEditorRef.current.setSelectionRange(start, end);
  }, [formalSourceEditorRef, activeFormalSourceSelection]);

  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">Formal QuantScript</div>
        <div className="property-card-caption">
          单独维护可编辑的 formal QuantScript 草稿，并作为 compile lane 的显式输入。
        </div>
      </div>
      <textarea
        ref={formalSourceEditorRef}
        aria-label="Formal QuantScript"
        value={formalQuantScriptSource}
        rows={14}
        onChange={(event) => updateFormalQuantScriptDraft(event.target.value)}
      />
      {formalApplyError ? (
        <div className="issue-row issue-error">
          <div className="issue-msg">{formalApplyError}</div>
        </div>
      ) : null}
      <div className="kv-line">
        <span>当前编译 formal source</span>
        <strong>{formalQuantScriptOverrideActive ? "已应用 override" : "图生成 formal source"}</strong>
      </div>
      <div className="toolbar-group">
        <button
          className="ad-btn ad-btn--ghost"
          onClick={() => {
            onActivateSourceLane?.();
            handleResetFormalQuantScript(setFormalApplyError);
          }}
        >
          重置 Formal QuantScript
        </button>
        <button
          className="ad-btn ad-btn--primary"
          onClick={() => {
            onActivateSourceLane?.();
            handleApplyFormalQuantScript(setFormalApplyError);
          }}
        >
          应用 Formal QuantScript
        </button>
      </div>
    </div>
  );
}

export function StrategyIrEditorCard({
  strategyIrSource,
  strategyIrEditorRef,
  selectedCompileDiagnosticTarget,
  strategyIrApplyError,
  updateStrategyIrDraft,
  handleResetStrategyIr,
  handleApplyStrategyIr,
  setStrategyIrApplyError,
  onActivateSourceLane = null
}) {
  return (
    <div className="property-card" data-testid="strategy-ir-editor-card">
      <div className="property-card-heading">
        <div className="property-card-title">策略中间表示 JSON</div>
        <div className="property-card-caption">保持语义预检工件可读，并允许直接编辑。</div>
      </div>
      <textarea
        ref={strategyIrEditorRef}
        aria-label="策略中间表示 JSON"
        value={strategyIrSource}
        rows={14}
        onChange={(event) => updateStrategyIrDraft(event.target.value)}
      />
      {selectedCompileDiagnosticTarget?.scope === "strategy_ir" ? (
        <div className="kv-line" data-testid="strategy-ir-focus-target">
          <span>当前聚焦目标</span>
          <strong>
            {selectedCompileDiagnosticTarget.label || selectedCompileDiagnosticTarget.field || "-"}
          </strong>
        </div>
      ) : null}
      {strategyIrApplyError ? (
        <div className="issue-row issue-error">
          <div className="issue-msg">{strategyIrApplyError}</div>
        </div>
      ) : null}
      <div className="toolbar-group">
        <button
          className="ad-btn ad-btn--ghost"
          onClick={() => {
            onActivateSourceLane?.();
            handleResetStrategyIr(setStrategyIrApplyError);
          }}
        >
          重置策略中间表示
        </button>
        <button
          className="ad-btn ad-btn--primary"
          onClick={() => {
            onActivateSourceLane?.();
            handleApplyStrategyIr(setStrategyIrApplyError);
          }}
        >
          应用策略中间表示
        </button>
      </div>
    </div>
  );
}

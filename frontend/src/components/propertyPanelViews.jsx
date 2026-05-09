import { useEffect, useRef, useState } from "react";
import { translateText, useI18n } from "../i18n";
import { StrategyCardNote } from "../pages/StrategyHubSharedComponents";
import {
  deriveConfigureCardOrder,
  derivePriorityFieldGroups,
  resolveConfigureIssueTargetCard
} from "../utils/configureFieldPriority";
import { getRuntimeStatusMeta, runtimeStatusLabel } from "../utils/runtimeStatus";
import DiagnosticsPanel from "./DiagnosticsPanel";
import RuntimeDiagnosticsPanel from "./RuntimeDiagnosticsPanel";
import {
  booleanStatusText,
  booleanStatusTone,
  compileOutputsText,
  diagnosticSeverityCounts,
  formatValue,
  runtimeSourceOfTruthText,
  runtimeSourceText,
  strategyIrRoleText
} from "../hooks/propertyPanelShared";
import { compileConflictGuidance, compileConflictSummary } from "../utils/compileContract";

function authoringKindLabel(kind) {
  if (kind === "risk") return "风控";
  if (kind === "execution") return "执行";
  if (kind === "data") return "数据";
  if (kind === "intent") return "意图";
  if (kind === "agent") return "代理";
  if (kind === "mixed") return "混合";
  if (kind === "unknown") return "未知";
  return kind || "-";
}

function authoringOriginLabel(origin) {
  if (origin === "authored") return "源码标注";
  if (origin === "derived") return "派生";
  if (origin === "hybrid") return "混合";
  return origin || "-";
}

function authoringStatusTone(status) {
  if (status === "ok") return "success";
  if (status === "mismatch") return "warning";
  if (status === "partial") return "muted";
  return "muted";
}

function authoringStatusLabel(status) {
  if (status === "ok") return "一致";
  if (status === "mismatch") return "不一致";
  if (status === "partial") return "部分";
  return status || "-";
}

function authoringRelationLabel(relation) {
  if (relation === "dataflow") return "数据流";
  if (relation === "decision_flow") return "决策流";
  if (relation === "policy_attachment") return "风控附着";
  if (relation === "execution_attachment") return "执行附着";
  return relation || "-";
}

function authoringPoolStageLabel(kind) {
  if (kind === "source") return "来源";
  if (kind === "eligibility") return "资格";
  if (kind === "features") return "特征";
  if (kind === "selection") return "选择";
  if (kind === "weighting") return "权重";
  if (kind === "rebalance") return "再平衡";
  return kind || "-";
}

function authoringPoolStageStatusTone(status) {
  if (status === "present") return "success";
  if (status === "empty") return "muted";
  return "muted";
}

function authoringPoolStageStatusLabel(status) {
  if (status === "present") return "已识别";
  if (status === "empty") return "留空";
  return status || "-";
}

function flowKindSummary(kind, sections) {
  const matchingSections = sections.filter((section) => section.effective_kind === kind);
  if (matchingSections.length === 0) {
    return {
      title: authoringKindLabel(kind),
      summary: "未识别",
      note: "当前 artifact 中没有这一段。"
    };
  }
  return {
    title: authoringKindLabel(kind),
    summary: `${matchingSections.length} 段`,
    note: matchingSections
      .map((section) => `${section.start_line}-${section.end_line}`)
      .join(", ")
  };
}

function lineRangeToSelection(source, startLine, endLine) {
  if (!source || !Number.isInteger(startLine) || !Number.isInteger(endLine)) {
    return null;
  }
  const normalizedStartLine = Math.max(1, startLine);
  const normalizedEndLine = Math.max(normalizedStartLine, endLine);
  const lines = source.split("\n");
  if (normalizedStartLine > lines.length) {
    return null;
  }

  let start = 0;
  for (let index = 0; index < normalizedStartLine - 1; index += 1) {
    start += lines[index].length + 1;
  }

  let end = start;
  for (
    let index = normalizedStartLine - 1;
    index < Math.min(normalizedEndLine, lines.length);
    index += 1
  ) {
    end += lines[index].length;
    if (index < lines.length - 1) {
      end += 1;
    }
  }

  return [start, Math.min(end, source.length)];
}

function sectionsToSelection(source, sections = []) {
  if (!sections.length) return null;
  const startLine = Math.min(...sections.map((section) => section.start_line));
  const endLine = Math.max(...sections.map((section) => section.end_line));
  return lineRangeToSelection(source, startLine, endLine);
}

export function renderFieldInput(field, value, onChange) {
  if (field.type === "select") {
    return (
      <select
        data-testid={`prop-input-${field.key || field.label}`}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      >
        {field.options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    );
  }

  if (field.type === "boolean") {
    return (
      <input
        data-testid={`prop-input-${field.key || field.label}`}
        type="checkbox"
        checked={Boolean(value)}
        onChange={(event) => onChange(event.target.checked)}
      />
    );
  }

  return (
    <input
      data-testid={`prop-input-${field.key || field.label}`}
      type={field.type === "number" ? "number" : "text"}
      value={value ?? ""}
      onChange={(event) =>
        onChange(field.type === "number" ? Number(event.target.value) : event.target.value)
      }
    />
  );
}

export function StatusChip({ tone, children }) {
  return <span className={`status-pill ${tone}`}>{children}</span>;
}

export function PropertySection({ kicker, title, summary, children, testId = null }) {
  return (
    <section
      className="property-section"
      aria-label={title}
      data-testid={testId || undefined}
    >
      <div className="property-section-header">
        {kicker ? <div className="property-section-kicker">{kicker}</div> : null}
        <div className="property-section-title">{title}</div>
        {summary ? <div className="property-section-summary">{summary}</div> : null}
      </div>
      <div className="property-section-body">{children}</div>
    </section>
  );
}

export function PropertySubsection({ title, children, testId = null }) {
  return (
    <div className="property-subsection" data-testid={testId || undefined}>
      <div className="property-subsection-title">{title}</div>
      <div className="property-subsection-body">{children}</div>
    </div>
  );
}

export function FieldGroup({ title, summary, children }) {
  return (
    <div className="property-field-group">
      <div className="property-field-group__header">
        <div className="property-field-group__title">{title}</div>
        {summary ? <div className="property-field-group__summary">{summary}</div> : null}
      </div>
      <div className="property-field-group__body">{children}</div>
    </div>
  );
}

export function PropertyPanelShell({ title, subtitle, children, className = "" }) {
  return (
    <aside className={`property-panel ${className}`.trim()}>
      <div className="property-panel-intro">
        <div className="panel-title strategy-card-title-note">
          <StrategyCardNote label={title} note={subtitle} />
        </div>
      </div>
      <div className="property-panel-scroll">{children}</div>
    </aside>
  );
}

export function WorkspaceInspectorShell({
  title,
  subtitle,
  summaryItems = [],
  actions = null,
  contextNotice = null,
  children
}) {
  return (
    <section className="workspace-section-card workspace-inspector-shell">
      <div className="workspace-section-card__header">
        <div>
          <div className="panel-title strategy-card-title-note">
            <StrategyCardNote label={title} note={subtitle} />
          </div>
        </div>
      </div>
      {contextNotice ? (
        <div className="workspace-inspector-shell__context">{contextNotice}</div>
      ) : null}
      {summaryItems.length > 0 || actions ? (
        <div className="workspace-inspector-shell__meta">
          {summaryItems.length > 0 ? (
            <div className="workspace-inspector-shell__summary">
              {summaryItems.map((item) => (
                <div
                  key={item.label}
                  className={`workspace-inspector-shell__summary-card${
                    item.tone ? ` workspace-inspector-shell__summary-card--${item.tone}` : ""
                  }`}
                >
                  <span>{item.label}</span>
                  <strong>{item.value}</strong>
                  {item.note ? <small>{item.note}</small> : null}
                </div>
              ))}
            </div>
          ) : null}
          {actions ? <div className="workspace-inspector-shell__actions">{actions}</div> : null}
        </div>
      ) : null}
      <div className="workspace-section-card__body">{children}</div>
    </section>
  );
}

export function QuantScriptAuthoringSourceCard({ authoringView }) {
  const activeSectionIds = new Set(authoringView?.activeSectionIds || []);
  const onSelectSection = authoringView?.onSelectSection || null;

  if (!authoringView) {
    return (
      <div className="property-card">
        <div className="property-card-heading">
          <div className="property-card-title">QuantScript 模块视图</div>
          <div className="property-card-caption">
            Formal 编译成功后，会在这里按源码顺序显示模块化工件。
          </div>
        </div>
        <div className="muted-line">当前还没有 quantscript_authoring_view。</div>
      </div>
    );
  }

  return (
    <div className="property-card" data-testid="qs-authoring-source-order">
      <div className="property-card-heading">
        <div className="property-card-title">QuantScript 模块视图</div>
        <div className="property-card-caption">
          按源码顺序展示后端生成的编写工件。
        </div>
      </div>
      <div className="muted-line">
        源码顺序：{authoringView.source_order.map(authoringKindLabel).join(" -> ")}
      </div>
      {authoringView.sections.map((section) => (
        <div
          key={section.id}
          className={`property-card${activeSectionIds.has(section.id) ? " property-card--active" : ""}`}
          data-testid={`authoring-section-${section.id}`}
        >
          <div className="property-card-heading">
            <div className="property-card-title">
              {authoringKindLabel(section.effective_kind)} · {section.id}
            </div>
            <div className="property-card-caption">
              L{section.start_line}-L{section.end_line}
            </div>
          </div>
          <button
            type="button"
            className="ghost-btn compact-btn"
            data-testid={`authoring-section-highlight-${section.id}`}
            onClick={() => onSelectSection?.([section.id])}
          >
            高亮源码范围
          </button>
          <div className="kv-line">
            <span>{t("声明类型")}</span>
            <strong>{authoringKindLabel(section.declared_kind)}</strong>
          </div>
          <div className="kv-line">
            <span>{t("实际类型")}</span>
            <strong>{authoringKindLabel(section.effective_kind)}</strong>
          </div>
          <div className="kv-line">
            <span>来源</span>
            <strong>{authoringOriginLabel(section.origin)}</strong>
          </div>
          <div className="kv-line">
            <span>状态</span>
            <strong>
              <StatusChip tone={authoringStatusTone(section.status)}>
                {authoringStatusLabel(section.status)}
              </StatusChip>
            </strong>
          </div>
          <div className="mini-list">
            <div className="mini-list-title">定义的符号</div>
            {section.symbols_defined.length === 0 ? (
              <div className="muted-line">没有显式定义。</div>
            ) : (
              section.symbols_defined.map((symbol) => (
                <div key={`${section.id}_defines_${symbol}`} className="mini-item">
                  {symbol}
                </div>
              ))
            )}
          </div>
          <div className="mini-list">
            <div className="mini-list-title">使用的符号</div>
            {section.symbols_used.length === 0 ? (
              <div className="muted-line">没有显式引用。</div>
            ) : (
              section.symbols_used.map((symbol) => (
                <div key={`${section.id}_uses_${symbol}`} className="mini-item">
                  {symbol}
                </div>
              ))
            )}
          </div>
          <textarea
            readOnly
            aria-label={`${section.id} snippet`}
            value={section.snippet}
            rows={Math.min(Math.max(section.snippet.split("\n").length + 1, 4), 12)}
          />
        </div>
      ))}
    </div>
  );
}

export function QuantScriptAuthoringStateCard({ authoringViewState }) {
  if (!authoringViewState || authoringViewState.mode !== "partial") {
    return null;
  }

  const errorCode = authoringViewState.error?.details?.[0]?.code || authoringViewState.error?.error || "-";
  const errorMessage =
    authoringViewState.error?.message || "Formal 编译失败，但编写工件仍以尽力方式保留。";

  return (
    <div className="property-card" data-testid="qs-authoring-partial-state">
      <div className="property-card-heading">
        <div className="property-card-title">QuantScript 编写回退</div>
        <div className="property-card-caption">
          Formal 编译失败时，源码工作区继续显示部分编写工件。
        </div>
      </div>
      <div className="kv-line">
        <span>状态</span>
        <strong>
          <StatusChip tone="warning">编译失败，已回退到部分工件</StatusChip>
        </strong>
      </div>
      <div className="kv-line">
        <span>错误代码</span>
        <strong>{errorCode}</strong>
      </div>
      <div className="muted-line">{errorMessage}</div>
    </div>
  );
}

export function QuantScriptAuthoringFlowCard({ authoringView }) {
  const activeSectionIds = new Set(authoringView?.activeSectionIds || []);
  const activeEdgeKey = authoringView?.activeEdgeKey || null;
  const onSelectSection = authoringView?.onSelectSection || null;
  const onSelectEdge = authoringView?.onSelectEdge || null;

  if (!authoringView) {
    return null;
  }

  return (
    <div className="property-card" data-testid="qs-authoring-pipeline-order">
      <div className="property-card-heading">
        <div className="property-card-title">QuantScript 流程视图</div>
        <div className="property-card-caption">
          按概念链路展示：数据 → 意图 → 代理 → 风控 → 执行。
        </div>
      </div>
      <div className="muted-line">
        管线顺序：{authoringView.pipeline_order.map(authoringKindLabel).join(" -> ")}
      </div>
      <div className="mini-list">
        <div className="mini-list-title">模块链路</div>
        {authoringView.pipeline_order.map((kind) => {
          const summary = flowKindSummary(kind, authoringView.sections);
          const matchingSection = authoringView.sections.find(
            (section) => section.effective_kind === kind
          );
          return (
            <button
              key={`pipeline_${kind}`}
              type="button"
              className={`mini-item${matchingSection && activeSectionIds.has(matchingSection.id) ? " mini-item--active" : ""}`}
              data-testid={`authoring-stage-${kind}`}
              onClick={() => {
                if (matchingSection) {
                  onSelectSection?.([matchingSection.id]);
                }
              }}
            >
              <strong>{summary.title}</strong>: {summary.summary}
              {summary.note ? ` (${summary.note})` : ""}
            </button>
          );
        })}
      </div>
      <div className="mini-list">
        <div className="mini-list-title">推导边</div>
        {authoringView.edges.length === 0 ? (
          <div className="muted-line">当前还没有推导出的模块连边。</div>
        ) : (
          authoringView.edges.map((edge) => {
            const edgeKey = `${edge.from}_${edge.to}_${edge.reason}`;
            return (
            <button
              key={edgeKey}
              type="button"
              className={`mini-item${activeEdgeKey === edgeKey ? " mini-item--active" : ""}`}
              data-testid={`authoring-edge-${edge.from}-${edge.to}`}
              onClick={() => onSelectEdge?.(edge)}
            >
              <strong>{edge.from}</strong> → <strong>{edge.to}</strong> ·{" "}
              {authoringRelationLabel(edge.relation)} · {edge.reason}
            </button>
          );
          })
        )}
      </div>
    </div>
  );
}

export function QuantScriptAuthoringPoolCard({ authoringView }) {
  const poolPipeline = authoringView?.pool_pipeline || null;
  const activeSectionIds = new Set(authoringView?.activeSectionIds || []);
  const onSelectSection = authoringView?.onSelectSection || null;

  if (!authoringView || !poolPipeline) {
    return null;
  }

  return (
    <div className="property-card" data-testid="qs-authoring-pool-pipeline">
      <div className="property-card-heading">
        <div className="property-card-title">标的池管线</div>
        <div className="property-card-caption">
          只读展示降级转换派生出的：来源 → 资格 → 特征 → 选择 → 权重 → 再平衡。
        </div>
      </div>
      <div className="muted-line">
        池顺序：{poolPipeline.order.map(authoringPoolStageLabel).join(" -> ")}
      </div>
      {poolPipeline.stages.map((stage) => {
        const isActive = stage.related_section_ids.some((id) => activeSectionIds.has(id));
        return (
          <div
            key={`pool-stage-${stage.kind}`}
            className={`property-card${isActive ? " property-card--active" : ""}`}
            data-testid={`authoring-pool-stage-${stage.kind}`}
          >
            <div className="property-card-heading">
              <div className="property-card-title">{authoringPoolStageLabel(stage.kind)}</div>
              <div className="property-card-caption">{stage.summary}</div>
            </div>
            <button
              type="button"
              className="ghost-btn compact-btn"
              data-testid={`authoring-pool-stage-highlight-${stage.kind}`}
              disabled={stage.related_section_ids.length === 0}
              onClick={() => {
                if (stage.related_section_ids.length > 0) {
                  onSelectSection?.(stage.related_section_ids);
                }
              }}
            >
              高亮关联模块
            </button>
            <div className="kv-line">
              <span>状态</span>
              <strong>
                <StatusChip tone={authoringPoolStageStatusTone(stage.status)}>
                  {authoringPoolStageStatusLabel(stage.status)}
                </StatusChip>
              </strong>
            </div>
            <div className="mini-list">
              <div className="mini-list-title">细节</div>
              {stage.details.length === 0 ? (
                <div className="muted-line">当前没有补充细节。</div>
              ) : (
                stage.details.map((detail) => (
                  <div key={`${stage.kind}_${detail}`} className="mini-item">
                    {detail}
                  </div>
                ))
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}

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

      <PropertySubsection title="Strategy IR 预检">
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
          <span>已生成 Core IR</span>
          <strong>
            <StatusChip tone={booleanStatusTone(strategyIrCheck?.has_core_ir)}>
              {booleanStatusText(strategyIrCheck?.has_core_ir)}
            </StatusChip>
          </strong>
        </div>
      </PropertySubsection>

      <PropertySubsection title="运行真源">
        <div className="kv-line">
          <span>Strategy IR 角色</span>
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
          className="ghost-btn"
          onClick={() => {
            onActivateSourceLane?.();
            handleResetQuantScript(setApplyError);
          }}
        >
          重置策略图源码
        </button>
        <button
          className="primary-btn"
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
          className="ghost-btn"
          onClick={() => {
            onActivateSourceLane?.();
            handleResetFormalQuantScript(setFormalApplyError);
          }}
        >
          重置 Formal QuantScript
        </button>
        <button
          className="primary-btn"
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
        <div className="property-card-title">Strategy IR JSON</div>
        <div className="property-card-caption">保持语义预检工件可读，并允许直接编辑。</div>
      </div>
      <textarea
        ref={strategyIrEditorRef}
        aria-label="Strategy IR JSON"
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
          className="ghost-btn"
          onClick={() => {
            onActivateSourceLane?.();
            handleResetStrategyIr(setStrategyIrApplyError);
          }}
        >
          重置 Strategy IR
        </button>
        <button
          className="primary-btn"
          onClick={() => {
            onActivateSourceLane?.();
            handleApplyStrategyIr(setStrategyIrApplyError);
          }}
        >
          应用 Strategy IR
        </button>
      </div>
    </div>
  );
}

export function NodeOverviewCard({ selectedNode, moduleDef, updateNodeName }) {
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">节点概览</div>
        <div className="property-card-caption">把节点身份、模块边界和可编辑名称收在同一处。</div>
      </div>
      <label className="field-block">
        <span>节点名称</span>
        <input
          data-testid="prop-input-node-name"
          value={selectedNode.name}
          onChange={(event) => updateNodeName(selectedNode.id, event.target.value)}
        />
      </label>
      <div className="kv-line">
        <span>模块</span>
        <strong>{moduleDef?.display_name || selectedNode.module_key}</strong>
      </div>
      <div className="kv-line">
        <span>模块键</span>
        <strong>{selectedNode.module_key}</strong>
      </div>
      <div className="kv-line">
        <span>类别</span>
        <strong>{moduleDef?.category || "-"}</strong>
      </div>
      <div className="kv-line">
        <span>节点 ID</span>
        <strong>{selectedNode.id}</strong>
      </div>
    </div>
  );
}

export function NodeConfigCard({
  selectedNode,
  moduleDef,
  updateNodeConfig,
  prioritizePathFields = false,
  nodeIssues = []
}) {
  const fieldGroups = derivePriorityFieldGroups({
    moduleDef,
    nodeIssues,
    nodeType: selectedNode?.type || null,
    prioritizePathFields
  });

  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">配置</div>
        <div className="property-card-caption">可编辑配置与运行状态、诊断信息保持分离。</div>
      </div>
      {(moduleDef?.config_schema?.fields || []).length === 0 ? (
        <div className="muted-line">这个节点当前没有可编辑配置项。</div>
      ) : null}
      {fieldGroups.map((group) => (
        <FieldGroup key={group.id} title={group.title} summary={group.summary}>
          {group.fields.map((field) => (
            <label key={field.key} className="field-block">
              <span>{field.label}</span>
              {renderFieldInput(field, selectedNode.config[field.key], (value) =>
                updateNodeConfig(selectedNode.id, field.key, value)
              )}
            </label>
          ))}
        </FieldGroup>
      ))}
    </div>
  );
}

export function ConnectionsCard({ graph, selectedNode }) {
  const incoming = graph.edges.filter((edge) => edge.target_node_id === selectedNode.id);
  const outgoing = graph.edges.filter((edge) => edge.source_node_id === selectedNode.id);

  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">连接关系</div>
        <div className="property-card-caption">在不离开节点面板的前提下查看上下游连线。</div>
      </div>
      <div className="mini-list">
        <div className="mini-list-title">输入</div>
        {incoming.length === 0 ? <div className="muted-line">当前没有输入边。</div> : null}
        {incoming.map((edge) => {
          const source = graph.nodes.find((node) => node.id === edge.source_node_id);
          return (
            <div key={edge.id} className="mini-item">
              {source?.name} -&gt; {edge.target_port}
            </div>
          );
        })}
      </div>
      <div className="mini-list">
        <div className="mini-list-title">输出</div>
        {outgoing.length === 0 ? <div className="muted-line">当前没有输出边。</div> : null}
        {outgoing.map((edge) => {
          const target = graph.nodes.find((node) => node.id === edge.target_node_id);
          return (
            <div key={edge.id} className="mini-item">
              {edge.source_port} -&gt; {target?.name}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function ValidationCard({ issues }) {
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">节点校验</div>
        <div className="property-card-caption">节点级问题紧贴配置显示，不与编译诊断混在一起。</div>
      </div>
      {issues.length === 0 ? <div className="muted-line">这个节点当前没有校验问题。</div> : null}
      {issues.map((issue) => (
        <div key={issue.id} className={`issue-row issue-${issue.level}`}>
          <div className="issue-msg">{issue.message}</div>
          {issue.hint ? <div className="issue-hint">{issue.hint}</div> : null}
        </div>
      ))}
    </div>
  );
}

function configureCardLabel(cardId) {
  if (cardId === "connections") return "连接";
  if (cardId === "config") return "配置";
  return "校验";
}

export function ActionableValidationCard({ issues, onSelectIssue = null }) {
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">节点校验</div>
        <div className="property-card-caption">节点级问题紧贴配置显示，不与编译诊断混在一起。</div>
      </div>
      {issues.length === 0 ? <div className="muted-line">这个节点当前没有校验问题。</div> : null}
      {issues.map((issue) => {
        const targetCardId = resolveConfigureIssueTargetCard(issue);
        const body = (
          <>
            <div className="issue-row__meta">
              <div className="issue-msg">{issue.message}</div>
              {onSelectIssue ? (
                <span className="diagnostic-chip diagnostic-chip--segment">
                  {configureCardLabel(targetCardId)}
                </span>
              ) : null}
            </div>
            {issue.hint ? <div className="issue-hint">{issue.hint}</div> : null}
          </>
        );

        if (!onSelectIssue) {
          return (
            <div key={issue.id} className={`issue-row issue-${issue.level}`}>
              {body}
            </div>
          );
        }

        return (
          <button
            key={issue.id}
            type="button"
            className={`issue-row issue-${issue.level} issue-row--actionable`}
            onClick={() => onSelectIssue(issue, targetCardId)}
          >
            {body}
          </button>
        );
      })}
    </div>
  );
}

export function NodeRuntimeCard({ selectedNode }) {
  const runtimeMeta = getRuntimeStatusMeta(selectedNode.runtime_state.status);
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">运行状态</div>
        <div className="property-card-caption">展示当前节点执行状态和最近一次可见运行信号。</div>
      </div>
      <div className="kv-line">
        <span>状态</span>
        <strong>
          <StatusChip tone={runtimeMeta.tone}>
            {runtimeStatusLabel(selectedNode.runtime_state.status)}
          </StatusChip>
        </strong>
      </div>
      <div className="kv-line">
        <span>最近事件</span>
        <strong>{selectedNode.runtime_state.last_event_type || "-"}</strong>
      </div>
      <div className="kv-line">
        <span>最近消息</span>
        <strong>{selectedNode.runtime_state.last_message || "-"}</strong>
      </div>
      <div className="kv-line">
        <span>最近时间</span>
        <strong>
          {selectedNode.runtime_state.last_event_time
            ? new Date(selectedNode.runtime_state.last_event_time).toLocaleTimeString()
            : "-"}
        </strong>
      </div>
    </div>
  );
}

export function NodeMetricsCard({ metrics }) {
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">运行指标</div>
        <div className="property-card-caption">原始节点指标与状态文本分开显示，阅读路径更清晰。</div>
      </div>
      {metrics.length === 0 ? <div className="muted-line">当前还没有运行指标。</div> : null}
      {metrics.map(([key, value]) => (
        <div key={key} className="kv-line">
          <span>{key}</span>
          <strong>{formatValue(value)}</strong>
        </div>
      ))}
    </div>
  );
}

export function NodeQuantScriptCard({ nodeSource }) {
  return (
    <div className="property-card">
      <div className="property-card-heading">
        <div className="property-card-title">节点源码工件</div>
        <div className="property-card-caption">只读展示当前节点对应的 graph-source 输出。</div>
      </div>
      <textarea readOnly value={nodeSource} rows={10} />
    </div>
  );
}

export function EdgeOverviewCard({ selectedEdge, sourceNode, targetNode, removeSelected }) {
  return (
    <div className="property-card">
      <div className="kv-line">
        <span>源节点</span>
        <strong>{sourceNode?.name || "-"}</strong>
      </div>
      <div className="kv-line">
        <span>目标节点</span>
        <strong>{targetNode?.name || "-"}</strong>
      </div>
      <div className="kv-line">
        <span>端口映射</span>
        <strong>
          {selectedEdge.source_port} -&gt; {selectedEdge.target_port}
        </strong>
      </div>
      <button className="danger-btn" onClick={removeSelected} data-testid="prop-action-delete-edge">
        删除边
      </button>
    </div>
  );
}

export function GraphConfigSection({ model }) {
  return (
    <PropertySection
      kicker="配置"
      title="策略图"
      summary="图的核心身份和问题计数保持在最上方。"
      testId="property-section-graph-config"
    >
      <GraphOverviewCard graph={model.graph} compileSummary={model.compileSummary} />
    </PropertySection>
  );
}

export function DiagnosticsSection({
  model,
  onRouteDiagnostic = null,
  graph = null,
  repairPathState = null
}) {
  return (
    <PropertySection
      kicker="编译"
      title="编译与诊断"
      summary="将编译结论、预检边界和结构化诊断收在一起。"
      testId="property-section-diagnostics"
    >
      <CompileSummaryCard compileSummary={model.compileSummary} />
      <DiagnosticsPanel
        compileSummary={model.compileSummary}
        onRouteDiagnostic={onRouteDiagnostic}
        graph={graph}
        repairPathState={repairPathState}
      />
    </PropertySection>
  );
}

export function SourceSection({
  model,
  includeNodeSource = false,
  onActivateSourceLane = null
}) {
  const [activeSectionIds, setActiveSectionIds] = useState([]);
  const [activeEdgeKey, setActiveEdgeKey] = useState(null);
  const [activeFormalSourceSelection, setActiveFormalSourceSelection] = useState(null);
  const graphSourceEditorRef = useRef(null);
  const formalSourceEditorRef = useRef(null);

  useEffect(() => {
    setActiveSectionIds([]);
    setActiveEdgeKey(null);
    setActiveFormalSourceSelection(null);
  }, [model.authoringView, model.formalQuantScriptSource]);

  function selectSections(sectionIds, edgeKey = null) {
    const sections = (model.authoringView?.sections || []).filter((section) =>
      sectionIds.includes(section.id)
    );
    if (!sections.length) return;
    onActivateSourceLane?.();
    setActiveSectionIds(sections.map((section) => section.id));
    setActiveEdgeKey(edgeKey);
    setActiveFormalSourceSelection(sectionsToSelection(model.formalQuantScriptSource, sections));
  }

  const authoringView = model.authoringView
    ? {
        ...model.authoringView,
        activeSectionIds,
        activeEdgeKey,
        onSelectSection: (sectionIds) => selectSections(sectionIds, null),
        onSelectEdge: (edge) =>
          selectSections([edge.from, edge.to], `${edge.from}_${edge.to}_${edge.reason}`)
      }
    : null;

  return (
    <PropertySection
      kicker="源码"
      title="脚本与 IR"
      summary="在不干扰编译结论的前提下编辑 graph-source 与 IR 工件。"
      testId="property-section-source"
    >
      {includeNodeSource ? <NodeQuantScriptCard nodeSource={model.nodeSource} /> : null}
      <QuantScriptAuthoringStateCard authoringViewState={model.authoringViewState} />
      <QuantScriptAuthoringSourceCard authoringView={authoringView} />
      <QuantScriptAuthoringFlowCard authoringView={authoringView} />
      <QuantScriptAuthoringPoolCard authoringView={authoringView} />
      <FormalQuantScriptEditorCard
        formalQuantScriptSource={model.formalQuantScriptSource}
        formalQuantScriptOverrideActive={model.formalQuantScriptOverrideActive}
        formalApplyError={model.formalApplyError}
        updateFormalQuantScriptDraft={model.updateFormalQuantScriptDraft}
        handleResetFormalQuantScript={model.handleResetFormalQuantScript}
        handleApplyFormalQuantScript={model.handleApplyFormalQuantScript}
        setFormalApplyError={model.setFormalApplyError}
        onActivateSourceLane={onActivateSourceLane}
        formalSourceEditorRef={formalSourceEditorRef}
        activeFormalSourceSelection={activeFormalSourceSelection}
      />
      <QuantScriptEditorCard
        graphSource={model.graphSource}
        sourceMode={model.graph.metadata.source_mode}
        applyError={model.applyError}
        updateQuantScriptDraft={model.updateQuantScriptDraft}
        handleResetQuantScript={model.handleResetQuantScript}
        handleApplyQuantScript={model.handleApplyQuantScript}
        setApplyError={model.setApplyError}
        onActivateSourceLane={onActivateSourceLane}
        graphSourceEditorRef={graphSourceEditorRef}
        activeSourceSelection={null}
      />
      <StrategyIrEditorCard
        strategyIrSource={model.strategyIrSource}
        strategyIrEditorRef={model.strategyIrEditorRef}
        selectedCompileDiagnosticTarget={model.selectedCompileDiagnosticTarget}
        strategyIrApplyError={model.strategyIrApplyError}
        updateStrategyIrDraft={model.updateStrategyIrDraft}
        handleResetStrategyIr={model.handleResetStrategyIr}
        handleApplyStrategyIr={model.handleApplyStrategyIr}
        setStrategyIrApplyError={model.setStrategyIrApplyError}
        onActivateSourceLane={onActivateSourceLane}
      />
    </PropertySection>
  );
}

export function NodeParamsSection({ model, prioritizePathFields = false }) {
  return (
    <PropertySection
      kicker="配置"
      title="节点设置"
      summary="将可编辑字段、校验和接线关系放进同一条设置泳道。"
      testId="property-section-node-params"
    >
      <NodeOverviewCard
        selectedNode={model.selectedNode}
        moduleDef={model.moduleDef}
        updateNodeName={model.updateNodeName}
      />
      <NodeConfigCard
        selectedNode={model.selectedNode}
        moduleDef={model.moduleDef}
        updateNodeConfig={model.updateNodeConfig}
        prioritizePathFields={prioritizePathFields}
        nodeIssues={model.nodeIssues}
      />
      <ConnectionsCard graph={model.graph} selectedNode={model.selectedNode} />
      <ValidationCard issues={model.nodeIssues} />
      <button className="danger-btn full-width" onClick={model.removeSelected} data-testid="prop-action-delete-node">
        删除节点
      </button>
    </PropertySection>
  );
}

export function LaneAwareNodeParamsSection({ model, prioritizePathFields = false }) {
  const [activeCardId, setActiveCardId] = useState(null);
  const cardRefs = useRef({});
  const selectedNodeId = model.selectedNode?.id || null;
  const cardOrder = deriveConfigureCardOrder({
    nodeIssues: model.nodeIssues,
    prioritizePathFields
  });

  useEffect(() => {
    setActiveCardId(null);
  }, [selectedNodeId]);

  useEffect(() => {
    if (!activeCardId) return;
    const cardNode = cardRefs.current[activeCardId];
    if (cardNode && typeof cardNode.scrollIntoView === "function") {
      cardNode.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, [activeCardId]);

  function handleSelectIssue(issue, targetCardId) {
    setActiveCardId(targetCardId || resolveConfigureIssueTargetCard(issue));
  }

  const orderedCards = {
    config: (
      <NodeConfigCard
        selectedNode={model.selectedNode}
        moduleDef={model.moduleDef}
        updateNodeConfig={model.updateNodeConfig}
        prioritizePathFields={prioritizePathFields}
        nodeIssues={model.nodeIssues}
      />
    ),
    connections: <ConnectionsCard graph={model.graph} selectedNode={model.selectedNode} />,
    validation: (
      <ActionableValidationCard issues={model.nodeIssues} onSelectIssue={handleSelectIssue} />
    )
  };

  return (
    <PropertySection
      kicker="配置"
      title="节点设置"
      summary="先固定节点身份，再根据当前问题类型重新排序配置、连接和校验区域。"
      testId="property-section-node-params"
    >
      <NodeOverviewCard
        selectedNode={model.selectedNode}
        moduleDef={model.moduleDef}
        updateNodeName={model.updateNodeName}
      />
      {cardOrder.map((cardId) => (
        <div
          key={cardId}
          ref={(node) => {
            cardRefs.current[cardId] = node;
          }}
          data-configure-card={cardId}
          className={`configure-card-anchor${
            activeCardId === cardId ? " configure-card-anchor--active" : ""
          }`}
        >
          {orderedCards[cardId]}
        </div>
      ))}
      <button className="danger-btn full-width" onClick={model.removeSelected} data-testid="prop-action-delete-node">
        删除节点
      </button>
    </PropertySection>
  );
}

export function NodeRuntimeSection({ model }) {
  return (
    <PropertySection
      kicker="运行"
      title="运行状态"
      summary="将当前节点状态与原始运行指标分开展示，便于快速扫读。"
      testId="property-section-node-runtime"
    >
      <NodeRuntimeCard selectedNode={model.selectedNode} />
      <RuntimeDiagnosticsPanel
        graph={model.graph}
        runtime={model.runtime}
        selectedNodeId={model.selectedNode?.id || null}
        title="节点运行诊断"
        subtitle="把当前节点的最近事件、输入输出快照和最近一次警告集中到同一张卡片。"
      />
      <NodeMetricsCard metrics={model.nodeMetrics} />
    </PropertySection>
  );
}


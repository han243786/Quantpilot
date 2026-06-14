import { useI18n } from "../i18n";
import { StatusChip } from "./propertyPanelLayoutPrimitives";

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

export function lineRangeToSelection(source, startLine, endLine) {
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

export function sectionsToSelection(source, sections = []) {
  if (!sections.length) return null;
  const startLine = Math.min(...sections.map((section) => section.start_line));
  const endLine = Math.max(...sections.map((section) => section.end_line));
  return lineRangeToSelection(source, startLine, endLine);
}

export function QuantScriptAuthoringSourceCard({ authoringView }) {
  const { t } = useI18n();
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
            className="ad-btn ad-btn--ghost compact-btn"
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
              className="ad-btn ad-btn--ghost compact-btn"
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

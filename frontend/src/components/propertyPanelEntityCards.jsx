import { formatValue } from "../hooks/propertyPanelShared";
import { derivePriorityFieldGroups, resolveConfigureIssueTargetCard } from "../utils/configureFieldPriority";
import { getRuntimeStatusMeta, runtimeStatusLabel } from "../utils/runtimeStatus";
import { FieldGroup, StatusChip, renderFieldInput } from "./propertyPanelLayoutPrimitives";

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
      <button className="ad-btn ad-btn--danger" onClick={removeSelected} data-testid="prop-action-delete-edge">
        删除边
      </button>
    </div>
  );
}

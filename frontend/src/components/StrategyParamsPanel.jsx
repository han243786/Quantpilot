import { usePropertyPanelModel } from "../hooks/usePropertyPanelModel";
import { buildRepairPathInsight } from "../utils/repairPathInsights";
import {
  EdgeOverviewCard,
  GraphConfigSection,
  LaneAwareNodeParamsSection,
  PropertySection,
  RepairPathContextPanel,
  WorkspaceInspectorShell
} from "./propertyPanelViews";

function configureRepairTarget(model) {
  if (model.selectedEdge?.id) {
    return {
      scope: "edge",
      edge_id: model.selectedEdge.id,
      label: `${model.sourceNode?.name || model.selectedEdge.source_node_id} -> ${
        model.targetNode?.name || model.selectedEdge.target_node_id
      }`
    };
  }

  if (model.selectedNode?.id) {
    return {
      scope: "node",
      node_id: model.selectedNode.id,
      label: model.selectedNode.name || model.selectedNode.id
    };
  }

  return null;
}

function configureRepairInsight(model, repairPathState) {
  const target = configureRepairTarget(model);
  const insight = buildRepairPathInsight(target, model.graph, repairPathState);
  if (!insight) return null;

  const changeNoun = model.selectedEdge ? "连线修改" : "配置修改";
  return {
    ...insight,
    note: insight.note.replaceAll("该项", changeNoun)
  };
}

export default function StrategyParamsPanel({ repairPathState = null }) {
  const model = usePropertyPanelModel();
  const repairPathInsight = configureRepairInsight(model, repairPathState);
  const prioritizePathFields =
    Boolean(repairPathInsight) && Boolean(model.selectedNode) && !Boolean(model.selectedEdge);

  const summaryItems = model.selectedEdge
    ? [
        {
          label: "当前选中",
          value: "边",
          note: `${model.sourceNode?.name || "-"} -> ${model.targetNode?.name || "-"}`
        },
        {
          label: "端口",
          value: `${model.selectedEdge.source_port} -> ${model.selectedEdge.target_port}`,
          note: "当前边映射。"
        }
      ]
    : model.selectedNode
      ? [
          {
            label: "当前选中",
            value: model.selectedNode.name || model.selectedNode.id,
            note: model.moduleDef?.display_name || model.selectedNode.type
          },
          {
            label: "问题数",
            value: model.nodeIssues.length,
            note: model.nodeIssues.length > 0 ? "需要处理校验问题。" : "当前没有节点级阻塞。"
          },
          {
            label: "运行态",
            value: model.selectedNode.runtime_state?.status || "idle",
            note: "节点最近一次已知运行状态。"
          }
        ]
      : [
          {
            label: "策略图",
            value: model.graph.metadata?.name || "草稿图",
            note: model.graph.metadata?.graph_id || "尚未生成图 ID。"
          },
          {
            label: "结构",
            value: `${model.graph.nodes.length} 个节点 / ${model.graph.edges.length} 条边`,
            note: "当前策略图结构。"
          },
          {
            label: "编译",
            value: model.compileSummary.compilable ? "可编译" : "待修复",
            note: "顶层策略图配置状态。"
          }
        ];

  const actions = model.selectedEdge ? (
    <button className="danger-btn compact-btn" onClick={model.removeSelected}>
      删除边
    </button>
  ) : model.selectedNode ? (
    <button className="danger-btn compact-btn" onClick={model.removeSelected}>
      删除节点
    </button>
  ) : null;

  const contextNotice = repairPathInsight ? (
    <RepairPathContextPanel
      insight={repairPathInsight}
      title={model.selectedEdge ? "当前连线上下文" : "当前变更上下文"}
      summary={
        model.selectedEdge
          ? "这次连线修改属于当前画布高亮的激活修复路径。"
          : "这次配置修改属于当前画布高亮的激活修复路径。"
      }
    />
  ) : null;

  if (model.selectedEdge) {
    return (
      <WorkspaceInspectorShell
        title="配置"
        subtitle="连线设置与节点、策略图控制保持分离。"
        summaryItems={summaryItems}
        actions={actions}
        contextNotice={contextNotice}
      >
        <PropertySection kicker="连线" title="边映射" summary="将边编辑与节点配置保持分离。">
          <EdgeOverviewCard
            selectedEdge={model.selectedEdge}
            sourceNode={model.sourceNode}
            targetNode={model.targetNode}
            removeSelected={model.removeSelected}
          />
        </PropertySection>
      </WorkspaceInspectorShell>
    );
  }

  return (
    <WorkspaceInspectorShell
      title="配置"
      subtitle="策略图身份、节点设置与结构操作控制。"
      summaryItems={summaryItems}
      actions={actions}
      contextNotice={contextNotice}
    >
      {model.selectedNode ? (
        <LaneAwareNodeParamsSection model={model} prioritizePathFields={prioritizePathFields} />
      ) : (
        <GraphConfigSection model={model} />
      )}
    </WorkspaceInspectorShell>
  );
}


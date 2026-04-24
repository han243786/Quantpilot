import { usePropertyPanelModel } from "../hooks/usePropertyPanelModel";
import { SourceSection, WorkspaceInspectorShell } from "./propertyPanelViews";

export default function StrategyCodePanel({ onActivateSourceLane = null }) {
  const model = usePropertyPanelModel();
  const summaryItems = [
    {
      label: "源码模式",
      value: model.graph.metadata?.source_mode || "graph",
      note: "当前源码真源模式。"
    },
    {
      label: "策略图源码",
      value: model.graphSource ? `${model.graphSource.length} 字符` : "空",
      note: "strategy_graph 文本工件。"
    },
    {
      label: "Formal QuantScript",
      value: model.formalQuantScriptSource ? `${model.formalQuantScriptSource.length} 字符` : "空",
      note: model.formalQuantScriptOverrideActive
        ? "当前编译将优先使用已应用的 formal source override。"
        : "当前编译默认使用图生成的 formal source。"
    },
    {
      label: "Strategy IR",
      value: model.strategyIrSource ? `${model.strategyIrSource.length} 字符` : "空",
      note: model.selectedNode
        ? `节点源码：${model.selectedNode.name || model.selectedNode.id}`
        : "策略图级 IR 编辑器。"
    },
    {
      label: "Authoring view",
      value: model.authoringView ? `${model.authoringView.sections?.length || 0} 段` : "未生成",
      note: model.authoringView
        ? "formal compile 产出的模块化源码视图。"
        : "需先完成一次 successful formal compile。"
    }
  ];

  const actions = (
    <>
      <button
        className="ghost-btn compact-btn"
        onClick={() => {
          onActivateSourceLane?.();
          model.handleResetQuantScript(model.setApplyError);
        }}
      >
        重置脚本
      </button>
      <button
        className="ghost-btn compact-btn"
        onClick={() => {
          onActivateSourceLane?.();
          model.handleResetStrategyIr(model.setStrategyIrApplyError);
        }}
      >
        重置 IR
      </button>
    </>
  );

  return (
    <WorkspaceInspectorShell
      title="源码"
      subtitle="策略图源码、Strategy IR 与节点级源码工件。"
      summaryItems={summaryItems}
      actions={actions}
    >
      <SourceSection
        model={model}
        includeNodeSource={Boolean(model.selectedNode)}
        onActivateSourceLane={onActivateSourceLane}
      />
    </WorkspaceInspectorShell>
  );
}


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
        ? "当前编译将优先使用已应用的 Formal 源码覆盖。"
        : "当前编译默认使用图生成的 Formal 源码。"
    },
    {
      label: "策略中间表示",
      value: model.strategyIrSource ? `${model.strategyIrSource.length} 字符` : "空",
      note: model.selectedNode
        ? `节点源码：${model.selectedNode.name || model.selectedNode.id}`
        : "策略图级 IR 编辑器。"
    },
    {
      label: "编写视图",
      value: model.authoringView ? `${model.authoringView.sections?.length || 0} 段` : "未生成",
      note: model.authoringView
        ? "Formal 编译产出的模块化源码视图。"
        : "需先完成一次成功的 Formal 编译。"
    }
  ];

  const actions = (
    <>
      <button
        className="ad-btn ad-btn--ghost compact-btn"
        onClick={() => {
          onActivateSourceLane?.();
          model.handleResetQuantScript(model.setApplyError);
        }}
      >
        重置脚本
      </button>
      <button
        className="ad-btn ad-btn--ghost compact-btn"
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
      subtitle="策略图源码、策略中间表示与节点级源码工件。"
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


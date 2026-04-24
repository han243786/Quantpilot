import { translateText } from "../i18n";
import { usePropertyPanelModel } from "../hooks/usePropertyPanelModel";
import {
  DiagnosticsSection,
  EdgeOverviewCard,
  GraphConfigSection,
  NodeParamsSection,
  NodeRuntimeSection,
  PropertyPanelShell,
  PropertySection,
  SourceSection
} from "./propertyPanelViews";

export default function PropertyPanel() {
  const model = usePropertyPanelModel();

  if (model.selectedEdge) {
    return (
      <PropertyPanelShell
        title="边详情"
        subtitle="单独查看连线映射，不把边元数据混进节点配置。"
      >
        <PropertySection
          kicker="连接"
          title="边映射"
          summary="把来源、目标和端口流向收在同一个紧凑视图里。"
        >
          <EdgeOverviewCard
            selectedEdge={model.selectedEdge}
            sourceNode={model.sourceNode}
            targetNode={model.targetNode}
            removeSelected={model.removeSelected}
          />
        </PropertySection>
      </PropertyPanelShell>
    );
  }

  if (!model.selectedNode) {
    return (
      <PropertyPanelShell
        title={translateText("策略图总览")}
        subtitle={translateText("从图配置、编译健康度和源码工件三条泳道阅读当前策略图。")}
      >
        <GraphConfigSection model={model} />
        <DiagnosticsSection model={model} />
        <SourceSection model={model} />
      </PropertyPanelShell>
    );
  }

  return (
    <PropertyPanelShell
      title="节点详情"
      subtitle="把节点状态拆成配置、编译健康度、运行活动和源码工件来阅读。"
    >
      <NodeParamsSection model={model} />
      <DiagnosticsSection model={model} />
      <NodeRuntimeSection model={model} />
      <SourceSection model={model} includeNodeSource />
    </PropertyPanelShell>
  );
}

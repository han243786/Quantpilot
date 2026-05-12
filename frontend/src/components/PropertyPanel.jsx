import { useState, useCallback } from "react";
import { translateText } from "../i18n";
import ErrorBoundary from "./ErrorBoundary";
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
  const [retryKey, setRetryKey] = useState(0);
  const handleRetry = useCallback(() => setRetryKey((k) => k + 1), []);

  if (model.selectedEdge) {
    return (
      <ErrorBoundary key={`edge-${retryKey}`} fallbackTitle="边详情加载失败" fallbackText="请刷新页面重试" onRetry={handleRetry}>
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
      </ErrorBoundary>
    );
  }

  if (!model.selectedNode) {
    return (
      <ErrorBoundary key={`overview-${retryKey}`} fallbackTitle="策略图总览加载失败" fallbackText="请刷新页面重试" onRetry={handleRetry}>
      <PropertyPanelShell
        title={translateText("策略图总览")}
        subtitle={translateText("从图配置、编译健康度和源码工件三条泳道阅读当前策略图。")}
      >
        <GraphConfigSection model={model} />
        <DiagnosticsSection model={model} />
        <SourceSection model={model} />
      </PropertyPanelShell>
      </ErrorBoundary>
    );
  }

  return (
    <ErrorBoundary key={`node-${retryKey}`} fallbackTitle="节点详情加载失败" fallbackText="请刷新页面重试" onRetry={handleRetry}>
    <PropertyPanelShell
      title="节点详情"
      subtitle="把节点状态拆成配置、编译健康度、运行活动和源码工件来阅读。"
    >
      <NodeParamsSection model={model} />
      <DiagnosticsSection model={model} />
      <NodeRuntimeSection model={model} />
      <SourceSection model={model} includeNodeSource />
    </PropertyPanelShell>
    </ErrorBoundary>
  );
}

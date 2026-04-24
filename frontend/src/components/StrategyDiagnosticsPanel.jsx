import { useMemo } from "react";
import { usePropertyPanelModel } from "../hooks/usePropertyPanelModel";
import { DiagnosticsSection, WorkspaceInspectorShell } from "./propertyPanelViews";

function diagnosticCounts(diagnostics = []) {
  return diagnostics.reduce(
    (summary, diagnostic) => {
      if (diagnostic?.severity === "warning") {
        summary.warning += 1;
      } else if (diagnostic?.severity === "info") {
        summary.info += 1;
      } else {
        summary.error += 1;
      }
      return summary;
    },
    { error: 0, warning: 0, info: 0 }
  );
}

export default function StrategyDiagnosticsPanel({
  onRouteDiagnostic = null,
  graph = null,
  repairPathState = null
}) {
  const model = usePropertyPanelModel();
  const counts = useMemo(
    () => diagnosticCounts(model.compileSummary?.diagnostics || []),
    [model.compileSummary]
  );

  const summaryItems = [
    {
      label: "可编译",
      value: model.compileSummary.compilable ? "是" : "否",
      note: model.compileSummary.backend_verified ? "已通过后端校验。" : "仅本地摘要。"
    },
    {
      label: "诊断",
      value: `${counts.error} / ${counts.warning} / ${counts.info}`,
      note: "错误 / 警告 / 提示"
    },
    {
      label: "协议",
      value: model.compileSummary.protocol_name || "-",
      note: model.compileSummary.config_hash || "尚未记录配置哈希。"
    }
  ];

  return (
    <WorkspaceInspectorShell
      title="校验"
      subtitle="编译结论、预检结果与结构化问题路由。"
      summaryItems={summaryItems}
    >
      <DiagnosticsSection
        model={model}
        onRouteDiagnostic={onRouteDiagnostic}
        graph={graph}
        repairPathState={repairPathState}
      />
    </WorkspaceInspectorShell>
  );
}


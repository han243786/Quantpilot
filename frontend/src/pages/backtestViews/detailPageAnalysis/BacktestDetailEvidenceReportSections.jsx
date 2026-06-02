import GovernedTimelinePanel from "../../../components/GovernedTimelinePanel";
import RuntimeReportPanel from "../../../components/RuntimeReportPanel";
import { AnalysisSection } from "../shared";

export function BacktestDetailGovernedTimelineSection({
  t = (value) => value,
  timelineSource = null
}) {
  return (
    <AnalysisSection
      testId="backtest-detail-governed-timeline"
      kicker={t("证据链")}
      title={t("治理时间轴")}
      summary={t("按 envelope 阶段、保留级别和模块查看回测证据，并优先保留关键事件。")}
    >
      <GovernedTimelinePanel
        source={timelineSource}
        title={t("回测证据时间轴")}
        summary={t("同一 timeline item 同时服务详情、回放、压缩证据和后续报告输入。")}
        testId="backtest-detail-timeline"
      />
    </AnalysisSection>
  );
}

export function BacktestDetailReportLifecycleSection({
  t = (value) => value,
  sourceId = "",
  timelineSource = null
}) {
  return (
    <AnalysisSection
      testId="backtest-detail-report-lifecycle"
      kicker={t("报告生命周期")}
      title={t("证据报告")}
      summary={t("从压缩证据生成可导出的报告，报告只链接来源证据和治理身份，不复制完整原始日志。")}
    >
      <RuntimeReportPanel
        sourceKind="backtest"
        sourceId={sourceId}
        evidenceSource={timelineSource}
        title={t("回测证据报告")}
        summary={t("生成、打开和导出当前回测的治理报告。")}
      />
    </AnalysisSection>
  );
}

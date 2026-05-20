/// v3.6.0: "部署到执行区"按钮 — 测试工作台一键部署，带阶段进度提示

import { useState, useRef } from "react";
import { useI18n } from "../i18n";
import { apiClient } from "../api/client";

const DEPLOY_STAGES = [
  { key: "compiling", label: "编译中..." },
  { key: "packing", label: "打包中..." },
  { key: "sending", label: "发送中..." },
  { key: "done", label: "完成" }
];

export default function DeployButton({ graph, canDeploy, onDeployed }) {
  const { t } = useI18n();
  const [deploying, setDeploying] = useState(false);
  const [deployStage, setDeployStage] = useState(null);
  const [result, setResult] = useState(null);
  const stageIndexRef = useRef(0);

  const simulateStage = () => {
    if (stageIndexRef.current < DEPLOY_STAGES.length - 1) {
      const nextIdx = stageIndexRef.current + 1;
      stageIndexRef.current = nextIdx;
      setDeployStage(DEPLOY_STAGES[nextIdx].key);
    }
  };

  const handleDeploy = async () => {
    if (!canDeploy) return;
    setDeploying(true);
    setResult(null);
    stageIndexRef.current = 0;
    setDeployStage(DEPLOY_STAGES[0].key);
    const stageTimer = setInterval(simulateStage, 800);
    try {
      const data = await apiClient.post("/executor/deploy", {
        graph_json: graph,
      });
      clearInterval(stageTimer);
      setDeployStage("done");
      setResult({ type: "success", message: t("策略已部署到执行器"), strategyId: data.strategy_id });
      onDeployed?.(data.strategy_id);
    } catch (err) {
      clearInterval(stageTimer);
      setDeployStage(null);
      const msg = err?.message || err?.error || String(err);
      setResult({ type: "error", message: msg.includes("连接") ? t("执行端未启动, 请先启动执行端") : msg });
    } finally {
      setDeploying(false);
    }
  };

  const stageLabel = deployStage
    ? DEPLOY_STAGES.find((s) => s.key === deployStage)?.label || t("部署中...")
    : null;

  return (
    <div style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
      <button
        className="primary-btn"
        onClick={handleDeploy}
        disabled={!canDeploy || deploying}
        title={!canDeploy ? t("请先编译策略") : t("部署到执行器 (127.0.0.1:3001)")}
        style={{ background: deploying ? "var(--exec-accent)" : undefined }}
      >
        {deploying ? (stageLabel || t("部署中...")) : t("部署到执行器")}
      </button>
      {result && (
        <span style={{ fontSize: 12, color: result.type === "success" ? "var(--ad-success)" : "var(--ad-error)" }}>
          {result.message}
        </span>
      )}
    </div>
  );
}

/// v3.4.0: 实时执行端 App Shell — 多策略标签页 + 状态轮询

import { useState, useEffect, useCallback, useRef } from "react";
import ExecutorTopBar from "./components/ExecutorTopBar";
import StrategyGraphPanel from "./components/StrategyGraphPanel";
import KlineChart from "./components/KlineChart";
import OrderPanel from "./components/OrderPanel";
import AssetPanel from "./components/AssetPanel";
import StrategyParamsPanel from "./components/StrategyParamsPanel";
import V4EvidencePanel from "./components/V4EvidencePanel";
import { useI18n } from "./i18n";

const API = "/api/executor";

export default function ExecutorApp() {
  const { t } = useI18n();
  const [strategies, setStrategies] = useState([]);
  const [activeTab, setActiveTab] = useState(null);
  const [execStatus, setExecStatus] = useState("idle"); // idle | running | error
  const [statusMsg, setStatusMsg] = useState(""); // v3.5.1: 错误详情
  const [mode, setMode] = useState("paper_simulated");
  const [modeError, setModeError] = useState(""); // v3.5.1: 模式切换错误
  const [strategyActionMessage, setStrategyActionMessage] = useState("");

  // v4.8.0: 加载当前双模拟盘执行模式
  useEffect(() => {
    fetch(`${API}/mode`).then(r => r.json()).then(d => {
      if (d.mode) setMode(d.mode);
    }).catch((e) => { console.warn("executor: mode load", e); });
  }, []);

  // v3.5.0: 模式切换处理
  const handleModeSwitch = useCallback(async (newMode) => {
    try {
      const res = await fetch(`${API}/mode`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ mode: newMode }),
      });
      if (res.ok) {
        const data = await res.json();
        setMode(data.current_mode);
        setModeError("");
        // 模式切换后刷新策略列表 (WS连接会重建)
        const stratRes = await fetch(`${API}/strategies`);
        if (stratRes.ok) {
          const stratData = await stratRes.json();
          setStrategies(stratData.strategies || []);
        }
      } else {
        const data = await res.json().catch(() => ({}));
        setModeError(data.message || t("模式切换失败"));
      }
    } catch (e) { console.warn("executor: mode switch", e); setModeError(t("模式切换失败，请检查后端连接")); }
  }, [t]);

  // 轮询活跃策略列表
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const res = await fetch(`${API}/strategies`);
        if (res.ok) {
          const data = await res.json();
          setStrategies(data.strategies || []);
          setExecStatus(data.strategies?.length > 0 ? "running" : "idle");
          if (!activeTab && data.strategies?.length > 0) {
            setActiveTab(data.strategies[0].strategy_id);
          }
        }
      } catch (e) { console.warn("executor: strategies poll", e); setExecStatus("error"); setStatusMsg(t("策略数据获取失败，请检查后端连接")); }
    }, 3000);
    return () => clearInterval(interval);
  }, [activeTab, t]);

  const activeStrategy = strategies.find(s => s.strategy_id === activeTab);
  const activeRuntimeKind = (activeStrategy?.runtime_kind || activeStrategy?.runtime_version || "v3").toLowerCase();

  // v3.4.0: 使用 ref 避免 useCallback 依赖 strategies 导致每3秒重建
  const strategiesRef = useRef(strategies);
  strategiesRef.current = strategies;
  const handleEmergencyStop = useCallback(async () => {
    if (!confirm(t("确认紧急停止所有策略并撤销全部挂单？"))) return;
    for (const s of strategiesRef.current) {
      await fetch(`${API}/strategies/${s.strategy_id}/stop`, { method: "POST" }).catch((e) => { console.warn("executor: emergency stop", e); });
    }
    setExecStatus("idle");
  }, [t]);

  const readExecutorError = useCallback(async (response) => {
    const text = await response.text().catch(() => "");
    try {
      const parsed = JSON.parse(text);
      return parsed.message || parsed.error || text || t("执行端操作失败");
    } catch (_) {
      return text || t("执行端操作失败");
    }
  }, [t]);

  const handleStrategyToggle = useCallback(async (strategy) => {
    const isRunning = (strategy.status || "").toLowerCase() === "running";
    const action = isRunning ? "stop" : "start";
    try {
      const response = await fetch(`${API}/strategies/${strategy.strategy_id}/${action}`, { method: "POST" });
      if (!response.ok) {
        const message = await readExecutorError(response);
        setExecStatus("error");
        setStrategyActionMessage(message);
        setStatusMsg(message);
        return;
      }
      setStrategyActionMessage("");
      setStatusMsg("");
      const stratRes = await fetch(`${API}/strategies`);
      if (stratRes.ok) {
        const stratData = await stratRes.json();
        setStrategies(stratData.strategies || []);
      }
    } catch (error) {
      console.warn("executor: start/stop", error);
      const message = t("执行端操作失败，请检查后端连接");
      setExecStatus("error");
      setStrategyActionMessage(message);
      setStatusMsg(message);
    }
  }, [readExecutorError, t]);

  return (
    <div className="exec-app">
      <ExecutorTopBar
        status={execStatus}
        statusMsg={statusMsg}
        strategyCount={strategies.length}
        onEmergencyStop={handleEmergencyStop}
        mode={mode}
        modeError={modeError}
        runtimeKind={activeRuntimeKind}
        onModeSwitch={handleModeSwitch}
      />

      <div className="exec-tabbar">
        {strategyActionMessage && (
          <div className="exec-tab" style={{ color: "var(--exec-error)" }}>
            {strategyActionMessage}
          </div>
        )}
        {strategies.map(s => (
          <div
            key={s.strategy_id}
            className={`exec-tab ${activeTab === s.strategy_id ? "active" : ""}`}
            onClick={() => setActiveTab(s.strategy_id)}
          >
            <span className={`exec-status-dot ${(s.status || "").toLowerCase() === "running" ? "running" : "stopped"}`} />
            {s.name || s.strategy_id}
            <span className="exec-runtime-pill">{(s.runtime_kind || s.runtime_version || "v3").toLowerCase()}</span>
            {/* v3.6.0 U10: 内联start/stop按钮 */}
            <button className="exec-tab-btn"
              onClick={(e) => { e.stopPropagation(); handleStrategyToggle(s); }}
              title={(s.status||"").toLowerCase()==="running" ? t("停止") : t("启动")}
            >{(s.status||"").toLowerCase()==="running" ? "⏹" : "▶"}</button>
          </div>
        ))}
        {strategies.length === 0 && (
          <div className="exec-tab" style={{ color: "var(--exec-text-secondary)" }}>
            {t("等待策略部署...")}
          </div>
        )}
      </div>

      <div className="exec-main">
        {activeStrategy ? (
          <>
            <div className="exec-sidebar">
              <StrategyParamsPanel strategyId={activeTab} />
              <V4EvidencePanel strategyId={activeTab} runtimeKind={activeRuntimeKind} />
              <OrderPanel strategyId={activeTab} />
              <AssetPanel strategyId={activeTab} />
            </div>
            <div className="exec-content">
              <div className="exec-graph-panel">
                <div className="exec-graph-locked-badge">🔒 {t("拓扑锁定 · 仅热调参")}</div>
                <StrategyGraphPanel strategy={activeStrategy} />
              </div>
              <div className="exec-kline-panel">
                <KlineChart strategyId={activeTab} />
              </div>
            </div>
          </>
        ) : (
          <div className="exec-empty">
            <div className="exec-empty-icon">⊞</div>
            <div className="exec-empty-text">{t("等待策略部署")}</div>
            <div style={{ fontSize: 12, color: "var(--exec-text-secondary)" }}>
              {t("在测试端编译策略后点击\"部署到执行区\"")}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

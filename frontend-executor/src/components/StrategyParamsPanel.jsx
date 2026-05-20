/// v3.5.0: 策略参数热调面板 — 运行时读取/修改/提交参数
/// GET /api/executor/params/:strategy_id → 渲染表单
/// POST /api/executor/params/:strategy_id → 提交更新

import { useState, useEffect, useCallback, memo } from "react";

const API = "/api/executor";

const ParamField = memo(function ParamField({ name, value, onChange }) {
  const type = typeof value;
  if (type === "boolean") {
    return (
      <div className="param-field">
        <label className="param-label">{name}</label>
        <label className="param-toggle">
          <input
            type="checkbox"
            checked={value}
            onChange={(e) => onChange(name, e.target.checked)}
          />
          <span>{value ? "开启" : "关闭"}</span>
        </label>
      </div>
    );
  }
  if (type === "number") {
    const isInteger = Number.isInteger(value);
    const abs = Math.abs(value);
    const min = isInteger ? 0 : 0;
    const max = isInteger ? Math.max(abs * 5, 100) : Math.max(abs * 5, 10);
    const step = isInteger ? 1 : abs > 1 ? 0.1 : 0.001;
    return (
      <div className="param-field">
        <label className="param-label">
          {name} <span className="param-value">{value}</span>
        </label>
        <div className="param-slider-row">
          <input
            type="range"
            min={min}
            max={max}
            step={step}
            value={value}
            onChange={(e) => onChange(name, parseFloat(e.target.value))}
          />
          <input
            type="number"
            className="param-number-input"
            value={value}
            step={step}
            onChange={(e) => {
              const v = parseFloat(e.target.value);
              if (!isNaN(v)) onChange(name, v);
            }}
          />
        </div>
      </div>
    );
  }
  return (
    <div className="param-field">
      <label className="param-label">{name}</label>
      <input
        type="text"
        className="param-text-input"
        value={value}
        onChange={(e) => onChange(name, e.target.value)}
      />
    </div>
  );
});

const StrategyParamsPanel = memo(function StrategyParamsPanel({ strategyId }) {
  const [params, setParams] = useState(null);
  const [edits, setEdits] = useState({});
  const [status, setStatus] = useState("idle"); // idle | pending | saving | saved | error

  useEffect(() => {
    if (!strategyId) return;
    setStatus("idle");
    const controller = new AbortController();
    fetch(`${API}/params/${strategyId}`, { signal: controller.signal })
      .then((r) => r.json())
      .then((d) => {
        setParams(d.params || {});
        setEdits({});
      })
      .catch((err) => {
        if (err.name !== "AbortError") setStatus("error");
      });
    return () => controller.abort(); // v3.5.1: 防止 strategyId 切换竞态
  }, [strategyId]);

  const handleChange = useCallback((name, value) => {
    setEdits((prev) => ({ ...prev, [name]: value }));
  }, []);

  const mergedParams = { ...params, ...edits };
  const hasChanges = Object.keys(edits).length > 0;

  const handleSubmit = useCallback(async () => {
    if (!hasChanges) return;
    setStatus("saving");
    try {
      const res = await fetch(`${API}/params/${strategyId}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ params: edits }),
      });
      if (res.ok) {
        setParams((prev) => ({ ...prev, ...edits }));
        setEdits({});
        setStatus("saved");
        setTimeout(() => setStatus("idle"), 3000);
      } else {
        setStatus("error");
      }
    } catch {
      setStatus("error");
    }
  }, [edits, hasChanges, strategyId]);

  const handleReset = useCallback(() => {
    setEdits({});
    setStatus("idle");
  }, []);

  if (!strategyId) {
    return <div className="params-panel-empty">请先部署策略</div>;
  }
  if (!params) {
    return <div className="params-panel-empty">加载参数中...</div>;
  }
  if (Object.keys(params).length === 0) {
    return <div className="params-panel-empty">该策略无可调参数</div>;
  }

  return (
    <div className="params-panel">
      <div className="params-header">
        <h3>策略参数</h3>
        <span className={`params-status params-status--${status}`}>
          {status === "saving" && "提交中..."}
          {status === "saved" && "已保存"}
          {status === "error" && "保存失败"}
          {status === "pending" && "有未提交的修改"}
          {status === "idle" && hasChanges && "有未提交的修改"}
        </span>
      </div>
      <div className="params-fields">
        {Object.entries(mergedParams).map(([key, value]) => (
          <ParamField key={key} name={key} value={value} onChange={handleChange} />
        ))}
      </div>
      <div className="params-actions">
        <button
          className="exec-btn primary"
          disabled={!hasChanges || status === "saving"}
          onClick={handleSubmit}
        >
          {status === "saving" ? "提交中..." : "提交参数"}
        </button>
        <button
          className="exec-btn"
          disabled={!hasChanges}
          onClick={handleReset}
        >
          重置
        </button>
      </div>
    </div>
  );
});

export default StrategyParamsPanel;

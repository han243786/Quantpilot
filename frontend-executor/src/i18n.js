import { createContext, createElement, useContext, useEffect, useMemo, useState } from "react";

const DEFAULT_LOCALE = "zh-CN";
const LOCALE_STORAGE_KEY = "quantpilot.locale";

const EN_US = {
  "QuantPilot 实时执行端": "QuantPilot Executor",
  "运行中": "Running",
  "错误": "Error",
  "空闲": "Idle",
  "策略": "strategies",
  "执行模式": "Execution mode",
  "实时模拟盘 / 本地撮合 / 无 provider 下单": "Paper simulation / local matching / no provider order submission",
  "OKX 模拟盘 / 非真实资金 / provider 回执": "OKX demo trading / no real funds / provider acknowledgements",
  "实时模拟盘": "Paper simulation",
  "OKX 模拟盘": "OKX demo",
  "紧急停止": "Emergency stop",
  "模式切换失败": "Mode switch failed",
  "模式切换失败，请检查后端连接": "Mode switch failed. Check the backend connection.",
  "策略数据获取失败，请检查后端连接": "Failed to load strategy data. Check the backend connection.",
  "确认紧急停止所有策略并撤销全部挂单？": "Emergency stop all strategies and cancel all open orders?",
  "停止": "Stop",
  "启动": "Start",
  "等待策略部署...": "Waiting for strategy deployment...",
  "拓扑锁定 · 仅热调参": "Topology locked · hot parameters only",
  "等待策略部署": "Waiting for strategy deployment",
  "在测试端编译策略后点击\"部署到执行区\"": "Compile a strategy in the test terminal, then deploy it to the executor.",
  "挂单": "Open orders",
  "暂无挂单": "No open orders",
  "策略部署到执行器后将在此显示订单状态。": "Order status appears here after a strategy is deployed.",
  "成交记录": "Fills",
  "买入": "Buy",
  "卖出": "Sell",
  "已创建": "Created",
  "已提交": "Submitted",
  "已接受": "Accepted",
  "部分成交": "Partially filled",
  "已成交": "Filled",
  "已取消": "Cancelled",
  "已拒绝": "Rejected",
  "资产": "Assets",
  "加载中...": "Loading...",
  "总权益": "Total equity",
  "现金余额": "Cash balance",
  "可用现金": "Available cash",
  "冻结": "Frozen",
  "持仓": "Positions",
  "等待策略加载...": "Waiting for strategy...",
  "图表加载中...": "Loading chart...",
  "图表范围": "Chart range",
  "图表周期与范围": "Chart timeframe and range",
  "数据源: public market-data / demo provider": "Source: public market data / demo provider",
  "请先部署策略": "Deploy a strategy first",
  "加载参数中...": "Loading parameters...",
  "该策略无可调参数": "This strategy has no tunable parameters",
  "策略参数": "Strategy parameters",
  "提交中...": "Submitting...",
  "已保存": "Saved",
  "保存失败": "Save failed",
  "有未提交的修改": "Unsaved changes",
  "提交参数": "Submit parameters",
  "重置": "Reset",
  "开启": "On",
  "关闭": "Off",
  "v4 状态机证据": "v4 state-machine evidence",
  "等待 v4 runtime memory_snapshot": "Waiting for v4 runtime memory_snapshot",
  "本地撮合": "Local matching",
  "provider 下单": "provider order submission",
  "无 provider 下单": "no provider order submission",
  "节点": "Node",
};

const dictionaries = {
  "zh-CN": {},
  "en-US": EN_US,
};

function normalizeLocale(locale) {
  if (!locale) return DEFAULT_LOCALE;
  if (dictionaries[locale]) return locale;
  return locale.toLowerCase().startsWith("en") ? "en-US" : DEFAULT_LOCALE;
}

function resolveInitialLocale() {
  if (typeof window === "undefined") return DEFAULT_LOCALE;
  const query = new URLSearchParams(window.location.search).get("lang");
  if (query) return normalizeLocale(query);
  try {
    return normalizeLocale(window.localStorage?.getItem(LOCALE_STORAGE_KEY));
  } catch (_) {
    return DEFAULT_LOCALE;
  }
}

const I18nContext = createContext({ locale: DEFAULT_LOCALE, t: (text) => text });

export function I18nProvider({ children }) {
  const [locale, setLocale] = useState(resolveInitialLocale);

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const value = useMemo(() => ({
    locale,
    setLocale: (next) => setLocale(normalizeLocale(next)),
    t(text) {
      return dictionaries[locale]?.[text] || text;
    },
  }), [locale]);

  return createElement(I18nContext.Provider, { value }, children);
}

export function useI18n() {
  return useContext(I18nContext);
}

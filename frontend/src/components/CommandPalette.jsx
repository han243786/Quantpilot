import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { useI18n } from "../i18n";
import { navigateTo } from "../router";
import { COMMAND_NAVIGATION_DEFS } from "../routing/shellNavigation";
import { useGraphStore } from "../store/graphStore";
import { humanizeErrorText } from "../utils/errorText";

const ACTION_COMMAND_DEFS = [
  { id: "save-graph", labelKey: "保存策略图", keys: ["save"], sectionKey: "操作", action: "saveGraph" },
  { id: "compile-graph", labelKey: "编译当前策略", keys: ["compile"], sectionKey: "操作", action: "compileCurrentGraph" },
  { id: "run-runtime", labelKey: "运行模拟", keys: ["run"], sectionKey: "操作", action: "startV4Simulation" },
  { id: "run-backtest", labelKey: "运行回测", keys: ["backtest"], sectionKey: "操作", action: "startBacktest" },
];
const COMMAND_DEFS = [...COMMAND_NAVIGATION_DEFS, ...ACTION_COMMAND_DEFS];

function toast(type, message) {
  if (typeof window === "undefined") return;
  window.dispatchEvent(new CustomEvent("qp-toast", { detail: { type, message } }));
}

export default function CommandPalette({ open, onClose }) {
  const { t } = useI18n();
  const COMMANDS = useMemo(() => COMMAND_DEFS.map((c) => ({
    ...c, label: t(c.labelKey), section: t(c.sectionKey)
  })), [t]);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef(null);
  const paletteRef = useRef(null);

  // v1.1.8: useMemo 防止每次渲染产生新数组引用
  const filtered = useMemo(() =>
    query.trim()
      ? COMMANDS.filter((c) =>
          c.label.toLowerCase().includes(query.toLowerCase()) ||
          c.section.toLowerCase().includes(query.toLowerCase()) ||
          c.keys.some((k) => k.includes(query.toLowerCase()))
        )
      : COMMANDS,
    [query, COMMANDS]
  );

  const select = useCallback((cmd) => {
    if (cmd.action) {
      const action = useGraphStore.getState()[cmd.action];
      if (typeof action !== "function") {
        toast("error", t("当前命令不可用"));
        onClose();
        return;
      }
      Promise.resolve(action())
        .then(() => toast("success", t("命令已完成")))
        .catch((error) => toast("error", humanizeErrorText(error?.message || String(error))));
      onClose();
      return;
    }
    navigateTo(cmd.keys[0]);
    onClose();
  }, [onClose, t]);

  useEffect(() => {
    if (!open) return;
    setQuery("");
    setSelectedIndex(0);
    const timer = setTimeout(() => inputRef.current?.focus(), 50);
    return () => clearTimeout(timer);
  }, [open]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  // v1.1.8: 用 ref 持有 filtered，避免 effect 依赖数组抖动
  const filteredRef = useRef(filtered);
  filteredRef.current = filtered;
  const selectedIndexRef = useRef(selectedIndex);
  selectedIndexRef.current = selectedIndex;

  useEffect(() => {
    if (!open) return;
    const handleKey = (e) => {
      const items = filteredRef.current;
      const idx = selectedIndexRef.current;
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((i) => Math.min(i + 1, items.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((i) => Math.max(i - 1, 0));
      } else if (e.key === "Enter" && items[idx]) {
        select(items[idx]);
      } else if (e.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [open, select, onClose]);

  // 全局 ⌘K / Ctrl+K 监听
  useEffect(() => {
    const handleGlobal = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        onClose ? (open ? onClose() : null) : null;
        if (typeof window.__toggleCommandPalette === "function") {
          window.__toggleCommandPalette();
        }
      }
    };
    window.addEventListener("keydown", handleGlobal);
    return () => window.removeEventListener("keydown", handleGlobal);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className="ad-overlay" onClick={onClose} data-testid="command-palette-overlay">
      <div
        className="ad-command-palette"
        ref={paletteRef}
        onClick={(e) => e.stopPropagation()}
        onKeyDown={(e) => {
          if (e.key === "Tab") {
            const items = paletteRef.current?.querySelectorAll("button");
            if (!items || items.length === 0) return;
            e.preventDefault();
            if (e.shiftKey) {
              if (document.activeElement === inputRef.current) {
                items[items.length - 1].focus();
              } else {
                const idx = Array.from(items).indexOf(document.activeElement);
                (items[idx - 1] || inputRef.current).focus();
              }
            } else {
              if (document.activeElement === items[items.length - 1]) {
                inputRef.current?.focus();
              } else {
                items[0]?.focus();
              }
            }
          }
        }}
        data-testid="command-palette"
      >
        <input
          ref={inputRef}
          className="ad-command-palette__input"
          placeholder={t("输入命令或搜索页面...")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <div className="ad-command-palette__list">
          {filtered.length === 0 ? (
            <div className="ad-command-palette__empty">{t("无匹配结果")}</div>
          ) : (
            filtered.map((cmd, i) => (
              <button
                key={cmd.id}
                className={`ad-command-palette__item${i === selectedIndex ? " ad-command-palette__item--selected" : ""}`}
                onClick={() => select(cmd)}
                onMouseEnter={() => setSelectedIndex(i)}
              >
                <span className="ad-command-palette__item-label">{cmd.label}</span>
                <span className="ad-command-palette__item-section">{cmd.section}</span>
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

// Global toggle
if (typeof window !== "undefined") {
  window.__toggleCommandPalette = null;
}

import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { useI18n } from "../i18n";
import { navigateTo } from "../router";

const COMMAND_DEFS = [
  { id: "strategies", labelKey: "策略中心", keys: ["/strategies"], sectionKey: "导航" },
  { id: "quantscript", labelKey: "QuantScript 编辑器", keys: ["/quantscript"], sectionKey: "导航" },
  { id: "approvals", labelKey: "审批队列", keys: ["/approvals"], sectionKey: "运维" },
  { id: "alerts", labelKey: "告警面板", keys: ["/alerts"], sectionKey: "运维" },
  { id: "snapshots", labelKey: "签名快照", keys: ["/snapshots"], sectionKey: "运维" },
  { id: "runbook", labelKey: "故障手册", keys: ["/runbook"], sectionKey: "运维" },
  { id: "chaos", labelKey: "混沌实验", keys: ["/chaos"], sectionKey: "运维" },
];

export default function CommandPalette({ open, onClose }) {
  const { t } = useI18n();
  const COMMANDS = useMemo(() => COMMAND_DEFS.map((c) => ({
    ...c, label: t(c.labelKey), section: t(c.sectionKey)
  })), [t]);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef(null);
  const paletteRef = useRef(null);

  const filtered = query.trim()
    ? COMMANDS.filter((c) =>
        c.label.toLowerCase().includes(query.toLowerCase()) ||
        c.section.toLowerCase().includes(query.toLowerCase()) ||
        c.keys.some((k) => k.includes(query.toLowerCase()))
      )
    : COMMANDS;

  const select = useCallback((cmd) => {
    navigateTo(cmd.keys[0]);
    onClose();
  }, [onClose]);

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

  useEffect(() => {
    if (!open) return;
    const handleKey = (e) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((i) => Math.min(i + 1, filtered.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((i) => Math.max(i - 1, 0));
      } else if (e.key === "Enter" && filtered[selectedIndex]) {
        select(filtered[selectedIndex]);
      } else if (e.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [open, filtered, selectedIndex, select, onClose]);

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

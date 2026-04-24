import { useEffect, useMemo, useState } from "react";
import {
  fetchBacktestReplay,
  fetchRunReplay
} from "../store/graphStoreRuntimeHistoryApi";
import { buildActionFailureMessage } from "../utils/actionFailure";
import { getEventExplanationSummary } from "../utils/runtimeExplanation";

const DEFAULT_PAGE_SIZE = 12;

function formatValue(value) {
  if (value === null || value === undefined || value === "") return "-";
  if (typeof value === "number") {
    return Number.isInteger(value) ? String(value) : value.toFixed(4);
  }
  return String(value);
}

function EventReplayRow({ item }) {
  const isFill = item.event?.event_type === "ExecutionFilled";
  const explanation = getEventExplanationSummary(item.event);
  return (
    <div
      className={`event-row${isFill ? " event-row-highlight" : ""}`}
      data-testid={`event-replay-row-${item.sequence_no}`}
    >
      <div className="event-time">#{item.sequence_no}</div>
      <div className="event-type">{item.event?.event_type || "-"}</div>
      <div className="event-details">
        <div className="event-message">{item.event?.summary || "-"}</div>
        {explanation ? <div className="muted-line">{explanation}</div> : null}
      </div>
    </div>
  );
}

export default function EventReplaySection({ runtime }) {
  const source = useMemo(() => {
    if (runtime?.selectedBacktestId) {
      return {
        kind: "backtest",
        id: runtime.selectedBacktestId,
        fetchReplay: fetchBacktestReplay,
        title: "事件回放",
        subtitle: "按页回看已持久化的回测事件、成交和账户摘要。"
      };
    }
    if (runtime?.selectedHistoryRunId) {
      return {
        kind: "run",
        id: runtime.selectedHistoryRunId,
        fetchReplay: fetchRunReplay,
        title: "事件回放",
        subtitle: "按页回看已持久化的运行事件、成交和账户摘要。"
      };
    }
    return null;
  }, [runtime?.selectedBacktestId, runtime?.selectedHistoryRunId]);

  const [pageSize, setPageSize] = useState(DEFAULT_PAGE_SIZE);
  const [cursor, setCursor] = useState(0);
  const [status, setStatus] = useState("idle");
  const [error, setError] = useState("");
  const [replay, setReplay] = useState(null);
  const [loadedOnce, setLoadedOnce] = useState(false);

  useEffect(() => {
    setPageSize(DEFAULT_PAGE_SIZE);
    setCursor(0);
    setStatus("idle");
    setError("");
    setReplay(null);
    setLoadedOnce(false);
  }, [source?.id, source?.kind]);

  async function loadReplay(nextCursor = 0) {
    if (!source) return;
    setStatus("loading");
    setError("");
    try {
      const payload = await source.fetchReplay(source.id, {
        cursor: nextCursor,
        limit: pageSize
      });
      setReplay(payload);
      setCursor(payload.cursor || 0);
      setStatus("ready");
      setLoadedOnce(true);
    } catch (loadError) {
      setStatus("error");
      setError(buildActionFailureMessage("event_replay", loadError, "加载事件回放失败。"));
    }
  }

  useEffect(() => {
    if (!loadedOnce) return;
    void loadReplay(0);
    // pageSize is the only intended trigger; source reset is handled above.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pageSize]);

  if (!source) return null;

  const summaryText = replay
    ? `${replay.cursor + 1}-${replay.window_end}/${replay.total_events}`
    : "未加载";

  return (
    <div className="open-orders-card" data-testid="event-replay-section">
      <div className="open-orders-header">
        <div>
          <div className="mini-list-title">{source.title}</div>
          <div className="muted-line">{source.subtitle}</div>
        </div>
        <strong data-testid="event-replay-window">{summaryText}</strong>
      </div>

      <div className="history-filter-row history-control-bar">
        <button
          type="button"
          className="ghost-btn compact-btn"
          data-testid="event-replay-load"
          disabled={status === "loading"}
          onClick={() => loadReplay(0)}
        >
          {loadedOnce ? "重新加载" : "加载回放"}
        </button>
        <select
          className="history-filter-input history-filter-select history-page-size-select"
          data-testid="event-replay-page-size"
          value={String(pageSize)}
          onChange={(event) => setPageSize(Number(event.target.value))}
        >
          <option value="6">每页 6 条</option>
          <option value="12">每页 12 条</option>
          <option value="20">每页 20 条</option>
        </select>
        <button
          type="button"
          className="ghost-btn compact-btn"
          data-testid="event-replay-prev"
          disabled={!replay?.previous_cursor && replay?.previous_cursor !== 0}
          onClick={() => loadReplay(replay.previous_cursor)}
        >
          上一页
        </button>
        <button
          type="button"
          className="ghost-btn compact-btn"
          data-testid="event-replay-next"
          disabled={!replay?.next_cursor && replay?.next_cursor !== 0}
          onClick={() => loadReplay(replay.next_cursor)}
        >
          下一页
        </button>
      </div>

      {status === "loading" ? <div className="muted-line">正在加载事件回放...</div> : null}
      {error ? <div className="history-note history-note-warning">{error}</div> : null}
      {!loadedOnce && status !== "loading" ? (
        <div className="muted-line">选择已持久化的运行或回测后，可按页查看事件回放。</div>
      ) : null}

      {replay ? (
        <>
          <div className="history-meta-grid">
            <div className="history-meta-chip history-meta-chip-wide">
              <span>记录</span>
              <strong>{replay.record_id}</strong>
            </div>
            <div className="history-meta-chip">
              <span>成交事件</span>
              <strong>{replay.fill_event_count}</strong>
            </div>
            <div className="history-meta-chip">
              <span>现金</span>
              <strong>{formatValue(replay.account?.cash_balance)}</strong>
            </div>
            <div className="history-meta-chip">
              <span>净值估算</span>
              <strong>{formatValue(replay.account?.equity_estimate)}</strong>
            </div>
          </div>

          {Array.isArray(replay.checkpoints) && replay.checkpoints.length > 0 ? (
            <div
              className="strategy-inspector-actions event-node-filter-bar"
              data-testid="event-replay-checkpoints"
            >
              {replay.checkpoints.slice(0, 6).map((checkpoint) => (
                <button
                  key={`${checkpoint.cursor}-${checkpoint.label}`}
                  type="button"
                  className={`ghost-btn compact-btn${
                    checkpoint.cursor === replay.cursor ? " is-active" : ""
                  }`}
                  data-testid={`event-replay-checkpoint-${checkpoint.cursor}`}
                  onClick={() => loadReplay(checkpoint.cursor)}
                >
                  {checkpoint.label}
                </button>
              ))}
            </div>
          ) : null}

          <div className="event-list" data-testid="event-replay-events">
            {replay.events.length === 0 ? (
              <div className="empty-state">当前回放窗口没有事件。</div>
            ) : null}
            {replay.events.map((item) => (
              <EventReplayRow key={item.sequence_no} item={item} />
            ))}
          </div>
        </>
      ) : null}
    </div>
  );
}

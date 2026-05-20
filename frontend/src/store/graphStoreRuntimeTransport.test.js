import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createRuntimeEventSource } from "./graphStoreRuntimeTransport";

describe("createRuntimeEventSource", () => {
  let mockInstances;

  beforeEach(() => {
    mockInstances = [];
    const MockEventSource = vi.fn(function (url) {
      const listeners = new Map();
      const instance = {
        url,
        listeners,
        addEventListener(type, handler) { listeners.set(type, handler); },
        close: vi.fn(function () { instance._manualClose = true; }),
        onerror: null,
        _manualClose: false,
        _reconnect: null,
        _onMessage: null,
        _onAccount: null,
        _onComplete: null,
        _onError: null,
        _reconnectTimer: null,
      };
      mockInstances.push(instance);
      return instance;
    });
    vi.stubGlobal("EventSource", MockEventSource);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("constructs EventSource with correct URL", () => {
    createRuntimeEventSource("run_test_1");
    expect(mockInstances[0].url).toContain("/runtime/runs/run_test_1/events");
  });

  it("prevents reconnect after manual close", () => {
    vi.useFakeTimers();
    const es = createRuntimeEventSource("run_001");
    expect(mockInstances).toHaveLength(1);

    es.close(); // sets _manualClose = true
    const result = es._reconnect();
    expect(result).toBeNull();
    expect(mockInstances).toHaveLength(1); // no new instance
    vi.useRealTimers();
  });

  it("forwards event listeners to reconnected instance", () => {
    vi.useFakeTimers();
    const es = createRuntimeEventSource("run_001");
    const msgHandler = vi.fn();
    const acctHandler = vi.fn();
    const completeHandler = vi.fn();
    const errorHandler = vi.fn();

    es._onMessage = msgHandler;
    es._onAccount = acctHandler;
    es._onComplete = completeHandler;
    es._onError = errorHandler;

    mockInstances[0]._reconnect?.();
    vi.advanceTimersByTime(1000);

    const next = mockInstances[1];
    expect(next.listeners.get("runtime_event")).toBe(msgHandler);
    expect(next.listeners.get("account")).toBe(acctHandler);
    expect(next.listeners.get("run_completed")).toBe(completeHandler);
    expect(next.onerror).toBe(errorHandler);
    vi.useRealTimers();
  });

  it("retries reconnect indefinitely without exhausting (v3.6.0 U9)", () => {
    vi.useFakeTimers();
    const onExhausted = vi.fn();
    createRuntimeEventSource("run_001", onExhausted);

    // 无限重连：不限次数 — 模拟10+次重连 onExhausted 也不应被调用
    for (let i = 0; i < 12; i++) {
      mockInstances[i]._reconnect?.();
      vi.advanceTimersByTime(Math.min(1000 * Math.pow(2, i), 60000));
    }
    expect(onExhausted).not.toHaveBeenCalled();
    vi.useRealTimers();
  });

});

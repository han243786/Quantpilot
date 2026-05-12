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

  it("calls onRetryExhausted after 5 reconnect attempts", () => {
    vi.useFakeTimers();
    const onExhausted = vi.fn();
    createRuntimeEventSource("run_001", onExhausted);

    for (let i = 0; i < 5; i++) {
      mockInstances[i]._reconnect?.();
      vi.advanceTimersByTime(1000 * Math.pow(2, i));
    }
    // 第6次 → retries>=MAX_RETRIES → exhausted
    const last = mockInstances[mockInstances.length - 1];
    last._reconnect?.();
    expect(onExhausted).toHaveBeenCalledTimes(1);
    vi.useRealTimers();
  });

});

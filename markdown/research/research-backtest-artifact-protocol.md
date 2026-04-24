# QuantPilot 研究级 Backtest 输入/输出工件协议深度研究

## 研究级工件协议的目标与约束

**事实（来自资料）**：在成熟研究平台里，“一次运行（run）”往往被视为一个可比较、可复现的最小单元：它需要记录参数、代码版本、指标与输出文件（artifacts），以便之后检索、对比与审计；例如 MLflow 明确将 run 定义为一次代码执行，并区分元数据（metrics、parameters、start/end time 等）与 artifacts（输出文件）。citeturn4view0  
**事实（来自资料）**：以 artifacts 为核心的系统通常强调“输入/输出谱系（lineage）”：同一个 run 会消费输入 artifacts、产出输出 artifacts，从而形成可追溯链路（W&B Artifacts 以“inputs/outputs of runs”的方式描述并支持版本与谱系图）。citeturn4view1turn0search17  
**事实（来自资料）**：数据版本化工具（如 DVC）强调把大数据与轻量元数据分离，以支持可复现与协作，并复用软件工程生态（Git + 远端存储/缓存）。citeturn4view2turn0search6  
**事实（来自资料）**：事件溯源（Event Sourcing）将“追加写事件日志”作为系统事实源（system of record），其他状态与视图通过回放事件派生；其优势包括审计性与历史重建，但也强调该模式复杂、迁移成本高、会约束未来设计，需要在收益足够时采用。citeturn4view3turn0search7turn0search3  

**借鉴（可迁移原则）**：QuantPilot 当前是单机 beta、标的与市场范围有限（paper / BTCUSDT / limited exchange），因此最适合借鉴的是“**单机可落地的最小可复现闭环**”，而不是分布式大平台。事件溯源的“日志为事实源 + 投影为派生结果”的结构非常契合你们强调的“统一协议、统一语义、可回放、可审计、可恢复”。citeturn4view3turn0search3  

**我的判断（QuantPilot 推荐）**：研究级 backtest 工件协议在 QuantPilot 的正确演化目标是：  
1) 在**协议层**把“输入（inputs）”定义成可哈希、可引用、可冻结（snapshot）的对象集合；  
2) 在**输出（outputs）**里把“事件流（event log）”确立为最小事实源，同时规定“成交账本/权益曲线/指标”必须能从事实源确定性派生；  
3) 用**ReproducibilityManifest**把“同输入可重放、同差异可解释”变成机器可验证的约束（不是人为约定）。

## 推荐协议总图与输入/输出边界定义

### 推荐协议总图

下面的总图强调：**RunSpec 绑定输入集合；EventLog 是事实源；Ledger/Equity/Metrics 是投影（可持久化以加速与便于 UI）**。

```
                 ┌─────────────────────────────────────────────┐
                 │                 RunSpec                      │
                 │  - run_id / engine_version / created_at      │
                 │  - pointers to inputs (digests + URIs)       │
                 │  - tags / comparison keys / seeds            │
                 └───────────────┬─────────────────────────────┘
                                 │ (declares)
        ┌────────────────────────┼────────────────────────┐
        │                        │                        │
┌───────▼────────┐     ┌─────────▼──────────┐    ┌────────▼──────────┐
│ BacktestSpec    │     │ MarketDataSnapshot │    │ ExecutionAssump.  │
│ (what to run)   │     │ (data frozen)      │    │ (fees/slip/lat)   │
└───────┬────────┘     └─────────┬──────────┘    └────────┬──────────┘
        │                        │                        │
        │                        │                        │
┌───────▼────────────────────────▼────────────────────────▼───────────┐
│                    Backtest Engine / Replay Runner                    │
│             (deterministic simulation with declared assumptions)      │
└───────┬──────────────────────────────────────────────────────────────┘
        │ produces (append-only, system of record)
┌───────▼──────────────┐
│ EventLogArtifact      │  ← 事实源：事件序列（市场/信号/订单/成交/资金）
└───────┬──────────────┘
        │ projections (deterministic derivations)
  ┌─────▼──────────────┐  ┌────────▼───────────┐  ┌────────▼──────────┐
  │ TradeLedgerArtifact  │  │ EquityCurveArtifact│  │ MetricsArtifact   │
  │ (fills/orders/pos)   │  │ (NAV time series)  │  │ (summary + ts)    │
  └─────────┬───────────┘  └─────────┬──────────┘  └─────────┬─────────┘
            │                          │                         │
            └──────────────┬───────────┴───────────┬────────────┘
                           ▼                       ▼
                 ReproducibilityManifest   Compare/Query Layer
                 (hash graph + env + diff) (API/UI/analysis)
```

**事实（来自资料）**：这种“输入/输出谱系”的表达与 W&B Artifacts 的“run 消费输入 artifacts、产出输出 artifacts”一致；而“节点+边”的元数据图表达也与 ML 元数据体系相似（Vertex ML Metadata 把 artifacts 与 executions 作为节点、events 作为连接输入/输出的边）。citeturn4view1turn5search22  

### 输入协议与输出协议的边界

**输入协议边界（QuantPilot Backtest Input Protocol）**  
“输入”必须是**足以重放**的一组声明对象与冻结对象，最小集合建议定义为：

- **声明类（declarative）**：BacktestSpec、ExecutionAssumptionSpec、RunSpec（含 seeds、比较标签、以及对所有输入的引用）。  
- **冻结类（snapshot / pinned）**：MarketDataSnapshotSpec（或 DatasetSpec + 明确的 snapshot 策略与 digest），以及 Strategy/Compile/CoreIR（可执行或可解释的策略表示）。  

**输出协议边界（QuantPilot Backtest Artifact Protocol）**  
“输出”必须包含一个**事实源**与一组可选/可派生投影：

- **事实源（必须）**：EventLogArtifact（追加写、可回放、可审计）。  
- **投影（建议默认产出并持久化）**：TradeLedgerArtifact、EquityCurveArtifact、MetricsArtifact。  
- **可复现锚点（必须）**：ReproducibilityManifest（把输入/输出连接成可验证哈希图）。  

**事实（来自资料）**：事件溯源模式下，事件存储作为 authoritative source，其他视图通过回放/投影得到；其核心价值就是审计与历史重建。citeturn4view3turn0search7  

## 输入协议建模与字段级建议

本节给出“字段分层（layering）”：所有 Spec 共享一个顶层 envelope（便于版本、哈希、兼容策略一致），再进入各自 domain 字段。

### 通用 Spec Envelope

**我的判断（QuantPilot 推荐）**：所有 *Spec* 都使用统一 envelope（JSON），并在存储时同时计算两类摘要：  
- `digest.bytes`：对规范化字节（canonical JSON）做 sha256；  
- `digest.semantic`：与 `digest.bytes` 一致（当 JSON 规范化严格时即可合并），用于“输入等价性判断”。  

**事实（来自资料）**：JSON 在不同序列化实现中可能出现 key 顺序、数值表示等差异，因此若要可靠哈希与签名，需要确定性规范化；RFC 8785 定义了 JSON Canonicalization Scheme（JCS）以得到可哈希的确定性表示。citeturn5search0  

建议 envelope（示例，省略次要字段）：

```json
{
  "kind": "BacktestSpec",
  "schema_version": "1.0.0",
  "protocol_version": "1.0.0",
  "id": "spec_backtest_...",
  "created_at": "2026-04-12T10:00:00Z",
  "created_by": {
    "actor": "user|system",
    "name": "alice"
  },
  "digest": {
    "alg": "sha256",
    "bytes": "sha256:..."
  },
  "compat": {
    "unknown_fields": "preserve",
    "deprecations": []
  },
  "spec": { }
}
```

### RunSpec / BacktestSpec

#### RunSpec

**事实（来自资料）**：在 run 为中心的追踪系统中，一次 run 会记录参数、指标、artifacts 等元信息，用于后续检索与对比。citeturn4view0turn4view1  

**我的判断（QuantPilot 推荐）**：QuantPilot 的 RunSpec 更像“可复现执行单元的声明”，字段应分为：身份、输入引用、比较维度、执行环境与确定性控制。

关键字段建议：

```json
{
  "kind": "RunSpec",
  "schema_version": "1.0.0",
  "protocol_version": "1.0.0",
  "id": "run_01J...", 
  "spec": {
    "experiment": {
      "experiment_id": "exp_btcusdt_beta",
      "name": "BTCUSDT-paper-beta",
      "group_keys": { "strategy": "mean_reversion_v3" }
    },
    "inputs": {
      "backtest_spec_ref": { "id": "spec_backtest_...", "digest": "sha256:..." },
      "dataset_snapshot_ref": { "id": "mdsnap_...", "digest": "sha256:..." },
      "execution_assumption_ref": { "id": "execasm_...", "digest": "sha256:..." },
      "strategy_ref": { "id": "strategy_...", "digest": "sha256:..." },
      "compile_ref": { "id": "compile_...", "digest": "sha256:..." },
      "core_ir_ref": { "id": "coreir_...", "digest": "sha256:..." }
    },
    "determinism": {
      "seed": 42,
      "rng_family": "pcg64",
      "float_mode": "strict|fast",
      "time_mode": "discrete_event",
      "threading": { "workers": 1 }
    },
    "engine": {
      "name": "QuantPilot",
      "engine_version": "0.1.0",
      "protocol_impl_version": "0.1.0",
      "build": { "git_commit": "..." }
    },
    "tags": {
      "venue": "paper",
      "symbol": "BTCUSDT",
      "data_freq": "1m",
      "note": "beta replay"
    }
  }
}
```

#### BacktestSpec

BacktestSpec 应只描述“研究意图与模拟边界”，而不混入“数据来源细节/费用细则”（这些分别属于 MarketDataSnapshotSpec 与 ExecutionAssumptionSpec）。  

必要字段（建议）：

- `universe`：标的集合（beta 可只支持单标的，但字段要支持扩展）。  
- `time_range`：`start_ts`, `end_ts`, `timezone_policy`（建议统一 UTC）。  
- `bar_spec`：粒度（tick/1m/5m）、价格字段（OHLC/last/mid/bid/ask）。  
- `portfolio`：初始资金、计价币种、杠杆/保证金模式（beta 可简化）。  
- `simulation_mode`：`replay` / `event_driven` / `vectorized`（即便现在只做 replay，也提前声明）。  
- `output_level`：输出工件级别（minimal / default / debug）。  

可选字段（建议）：

- `risk_constraints`（如最大仓位、最大下单频率）。  
- `market_calendar`（对 crypto 可默认 24/7，但仍要声明）。  
- `benchmark`（用于 alpha/beta 等指标）。  

### DatasetSpec / MarketDataSnapshotSpec

**事实（来自资料）**：可复现体系通常要求对数据版本进行管理，并通过元数据指向大文件内容（DVC 的理念是把大数据与轻量元数据分离，从而实现可复现）。citeturn4view2turn0search6  

**我的判断（QuantPilot 推荐）**：QuantPilot 早期不追求市场全覆盖，反而应把关键点做“硬”：**每次 backtest 必须明确使用了哪一份数据快照**。因此建议把 “DatasetSpec（数据集定义）” 与 “MarketDataSnapshotSpec（本次运行实际使用的数据切片冻结）”区分开：

- DatasetSpec：可以是“逻辑数据集”（例如 Binance BTCUSDT 1m candles，按日期分区，持续增长）。  
- MarketDataSnapshotSpec：是“本 run 实际读取的数据切片”，应可被完全重建，或直接内嵌为文件快照（单机 beta 推荐直接复制/裁剪成 run 内快照，避免外部依赖漂移）。

MarketDataSnapshotSpec 必要字段建议：

```json
{
  "kind": "MarketDataSnapshotSpec",
  "schema_version": "1.0.0",
  "id": "mdsnap_...",
  "spec": {
    "dataset": {
      "dataset_id": "binance_btcusdt_candles_1m",
      "provider": "binance",
      "instrument": "BTCUSDT",
      "schema": {
        "format": "parquet",
        "columns": [
          {"name":"ts","type":"timestamp[ms,UTC]"},
          {"name":"open","type":"decimal128(38,18)"},
          {"name":"high","type":"decimal128(38,18)"},
          {"name":"low","type":"decimal128(38,18)"},
          {"name":"close","type":"decimal128(38,18)"},
          {"name":"volume","type":"decimal128(38,18)"}
        ]
      }
    },
    "slice": {
      "start_ts": "2024-01-01T00:00:00Z",
      "end_ts": "2024-03-01T00:00:00Z",
      "bar_freq": "1m",
      "filters": {"venue":"paper"}
    },
    "snapshot_strategy": {
      "mode": "materialized_copy",
      "files": [
        {
          "role": "candles",
          "path": "artifacts/inputs/market_data/candles.parquet",
          "digest": "sha256:...",
          "bytes": 123456789,
          "row_count": 86400
        }
      ]
    },
    "data_quality": {
      "missing_bars_policy": "explicit_gap_events",
      "dedup_policy": "stable_sort_then_drop_dups",
      "timezone_policy": "force_utc"
    }
  }
}
```

### ExecutionAssumptionSpec

**事实（来自资料）**：现实建模（fees/slippage/fill model）是回测可信度的核心差异来源；例如常见回测平台强调可插拔的 fee/slippage/fill 模型，以控制回测乐观/悲观程度。citeturn3search33  

**我的判断（QuantPilot 推荐）**：ExecutionAssumptionSpec 的目标是让“假设差异可追踪、可解释”，因此必须把费用、滑点、延迟、撮合与舍入等写成**可比较的结构化参数**，并提供一个可展示的 `assumption_fingerprint`（由 canonical JSON digest 得到）。

必要字段（建议）：

- `fee_model`：maker/taker、按成交额比例、最低费用、币种。  
- `slippage_model`：价差、冲击、滑点上限、与 bar/tick 的关系。  
- `latency_model`：下单/撤单延迟、撮合延迟（离散事件模拟）。  
- `fill_model`：成交优先级、部分成交、是否允许同 bar 成交（避免未来函数）。  
- `market_rules`：最小下单量、步进、tick size、最大挂单数（即便 beta 简化，也要声明默认值）。  
- `valuation_policy`：权益曲线使用 last/mid/close 哪个价格、费用入账时间点、资金费率/隔夜费（crypto 可选）。  

可选字段（建议）：

- `borrow/lending`、`funding_rate_model`、`liquidation_model`（未来增强）。  

## 输出工件协议与 Artifact Taxonomy

### Artifact taxonomy：一等公民 vs 派生结果

**事实（来自资料）**：在事件溯源体系里，“事件日志”是事实源；而为了可查询，通常会维护投影/物化视图，这些视图是从事件派生但可被持久化。citeturn4view3  

**我的判断（QuantPilot 推荐）**：QuantPilot 的回测工件可分三层：

- **一等公民（必须持久化，作为可复现与审计锚点）**  
  - RunSpec（含全部输入引用）  
  - BacktestSpec  
  - MarketDataSnapshotSpec（以及快照文件本体，或可验证的外部引用）  
  - ExecutionAssumptionSpec  
  - StrategyArtifact / CompileArtifact / CoreIRArtifact（至少要能证明策略“是什么”）  
  - EventLogArtifact（事实源）  
  - ReproducibilityManifest（哈希图与环境声明）

- **高价值投影（强烈建议持久化；可从事实源重建，但能显著降低 UI/分析成本）**  
  - TradeLedgerArtifact  
  - EquityCurveArtifact  
  - MetricsArtifact（尤其是 summary + returns time series）

- **派生/可选（debug 级别，beta 可延后或按需）**  
  - Step-level state snapshots、profiling traces、orderbook microstructure 细节、策略内部调试变量全量 dump。

### 通用 Artifact Envelope 与存储拆分

**借鉴（可迁移原则）**：像 W&B 和 MLflow 这类系统通常把“元数据（可查询）”与“文件（artifacts）”分离，run 记录元数据并在 artifact store 中存放文件。citeturn4view0turn4view1  

**我的判断（QuantPilot 推荐）**：每个 Artifact = `artifact_meta.json` + `payload files`。元数据统一 envelope：

```json
{
  "artifact_type": "EventLogArtifact",
  "schema_version": "1.0.0",
  "protocol_version": "1.0.0",
  "artifact_id": "art_evlog_...",
  "run_id": "run_01J...",
  "created_at": "2026-04-12T10:01:00Z",
  "producer": {"engine_version":"0.1.0"},
  "digest": {"alg":"sha256", "bytes":"sha256:..."},
  "links": {
    "inputs": [
      {"kind":"MarketDataSnapshotSpec","id":"mdsnap_...","digest":"sha256:..."},
      {"kind":"ExecutionAssumptionSpec","id":"execasm_...","digest":"sha256:..."}
    ],
    "derived_from": [{"artifact_type":"EventLogArtifact","artifact_id":"..."}]
  },
  "payload": {
    "format": "parquet|ndjson|arrow_ipc",
    "files": [{"path":"artifacts/outputs/event_log/events.parquet","digest":"sha256:..."}]
  }
}
```

### 各工件字段级建议与“必要/可选”划分

下表述按“当前 beta 最小实现”优先级给出。

#### StrategyArtifact / CompileArtifact / CoreIRArtifact

**我的判断（QuantPilot 推荐）**：三者解决三个问题：  
- StrategyArtifact：策略“语义身份”（源码/配置/参数）。  
- CompileArtifact：策略“可执行物”（二进制/wasm/解释器字节码）。  
- CoreIRArtifact：策略“可比较表达”（用于 diff、审计与跨版本兼容）。  

必要字段（建议）：

- StrategyArtifact：`strategy_name`、`entrypoint`、源码包 hash、配置参数（canonical JSON）、依赖锁文件 hash。  
- CompileArtifact：编译器版本、编译 flags、产物 hash、build 环境摘要。  
- CoreIRArtifact：IR 版本、IR 内容 hash、生成器版本、与源码的映射（source map 可选）。  

**事实（来自资料）**：可复现执行单元常用“输入完全决定输出”的 digest 标识；例如 Bazel Remote Execution API 中，一个 Action 被定义为包含复现所需信息，并可由其编码后的 digest 唯一标识。citeturn2search18turn2search14  

#### EventLogArtifact（事实源）

**事实（来自资料）**：事件溯源强调追加写事件序列，并能通过回放重建状态与历史。citeturn4view3turn0search7  
**借鉴（可迁移原则）**：可观测性体系（OpenTelemetry）对“事件记录”常采用统一字段：Timestamp、ObservedTimestamp、TraceId/SpanId、Attributes 等，以便关联与查询。citeturn3search15turn3search3  

**我的判断（QuantPilot 推荐）**：EventLog 必须可作为“唯一事实源”，因此对事件的 **时间语义** 与 **关联 ID** 要硬性规定。

必要字段（每条 event）：

- `seq`：单调递增序号（run 内全局）。  
- `ts_event`：事件发生时间（市场数据时间/撮合时间/资金时间，统一 UTC）。  
- `ts_observed`：引擎观察/处理时间（用于调试与延迟分析）。  
- `event_type`：枚举（见下）。  
- `trace`：`trace_id`, `span_id`（可选但强烈建议，便于把“策略决策→订单→成交”串起来）。citeturn3search3turn3search15  
- `keys`：`instrument`, `venue`, `order_id`, `client_order_id`, `fill_id`, `position_id` 等。  
- `payload`：事件类型对应结构体（必须 schema 化，避免自由文本）。  

事件类型最小集合（beta 版建议）：

- `MarketDataBar`（或 `MarketDataTick`）：bar/tick 输入。  
- `SignalEmitted`：策略输出的信号（不等于下单）。  
- `OrderRequested` / `OrderAccepted` / `OrderRejected`  
- `OrderCanceled` / `OrderCancelRejected`  
- `Fill`（成交）  
- `FeeCharged`（也可并入 Fill）  
- `PositionChanged`（可选：也可从 Fill 派生，但记录能简化审计）  
- `EquitySample`（可选：建议作为投影，不作为事实源）  

#### TradeLedgerArtifact（成交账本）

**事实（来自资料）**：许多交易系统用标准字段区分“客户端订单 ID / 场内订单 ID / 成交回报 ID”；例如 FIX Execution Report 提到 ClOrdID（客户端订单标识）、OrderID（卖方生成订单号）等。citeturn1search3turn1search27  

**我的判断（QuantPilot 推荐）**：TradeLedger 应是从 EventLog 的 `Fill/Fee/Order*` 事件确定性投影而来，且要能支撑：  
- UI 的交易列表与订单生命周期；  
- 指标计算（胜率、交易频率、平均滑点、费用占比）；  
- “回放差异解释”（执行假设不同导致 fill 不同）。

最小字段集合（fills 表）：

- 标识：`fill_id`, `order_id`, `client_order_id`, `trade_id`（可按策略定义聚合）。  
- 时间：`ts_fill`, `ts_order_submit`, `ts_order_ack`（用于延迟/排队分析）。  
- 合约：`instrument`, `venue`, `side`, `qty`, `price`, `notional`。  
- 成本：`fee_amount`, `fee_currency`, `slippage_bps`（或 `expected_price` vs `fill_price`）。  
- 归因：`signal_id`（关联 SignalEmitted）、`trace_id`。  

可选（研究增强）：

- 订单簿/盘口快照引用（用于微观结构研究）。  
- 成交分解（partial fills）。  

#### EquityCurveArtifact（权益曲线）

**事实（来自资料）**：主流回测平台的结果页通常至少包含权益曲线、交易列表、日志与性能统计。citeturn3search1  

**我的判断（QuantPilot 推荐）**：权益曲线必须严格定义其计算口径（valuation_policy），否则“同输入可重放”会在细节上失效（尤其是价源与舍入）。ExecutionAssumptionSpec 里必须声明 equity 估值用的 price（close/mid/last）、费用/资金费入账时点等。

最小字段集合：

- `ts`（采样时间点）  
- `equity`（净值）  
- `cash`  
- `position_value`  
- `unrealized_pnl`, `realized_pnl`  
- `fees_cum`  
- `drawdown`（可选：也可由 equity 派生但常用）  
- `num_positions`, `gross_exposure`, `net_exposure`（可选，增强解释性）

#### MetricsArtifact（指标汇总）

**借鉴（可迁移原则）**：实验追踪体系通常把 metrics 当作一等公民的可查询对象（例如 MLflow run 记录 metrics；W&B 也在 run 中记录 metrics，并把 artifacts 用于版本化输入输出）。citeturn4view0turn4view1  

**我的判断（QuantPilot 推荐）**：MetricsArtifact 应同时包含：  
1) `summary`（标量汇总，用于列表页排序/过滤/比较）；  
2) `timeseries`（最少 returns 序列，用于可复算关键风险指标）；  
3) `assumption_attribution`（把关键假设参数镜像进来，便于对比时展开）。

最小字段（summary）建议：

- 收益类：`total_return`, `cagr`（或 crypto 以年化/期间化方式声明）  
- 风险类：`max_drawdown`, `volatility`, `sharpe`, `sortino`  
- 交易类：`num_trades`, `win_rate`, `avg_trade_return`, `turnover`  
- 成本类：`fee_total`, `avg_slippage_bps`  

#### ReproducibilityManifest（可复现清单）

**事实（来自资料）**：内容寻址系统通过对内容哈希来标识对象（Git 的对象以内容哈希命名；Nix 也支持内容寻址的 store objects），从而在“引用→内容”层面提供强一致性。citeturn2search1turn2search7  
**事实（来自资料）**：JSON 若用于哈希签名，需确定性规范化（RFC 8785）。citeturn5search0  

**我的判断（QuantPilot 推荐）**：Manifest 是“同输入可重放”的机器证明：它是一张哈希图（hash graph）与环境声明，最少要覆盖：

- `run_id` 与 `run_spec_digest`  
- 所有输入 spec/inputs payload 的 digest 列表  
- 所有输出 artifacts 的 digest 列表  
- 引擎与依赖版本、操作系统/CPU 信息、编译器版本  
- 确定性配置（seed、线程数、浮点模式）  
- `replay_contract`：声明“只要这些 digest 不变，重放结果应一致；如不一致则为 bug/非确定性来源”  

## 协议关系：事件流、成交账本、权益曲线、指标汇总如何闭环

这一节专门回答你强调的“事件流 + 权益曲线 + 成交账本 + 指标汇总”之间的协议关系，并给出可审计、可回放的依赖链。

### 关系定义：事实源与投影

**事实（来自资料）**：事件溯源体系把事件序列作为 system of record，并通过回放构建当前状态与物化视图。citeturn4view3turn0search7  

**我的判断（QuantPilot 推荐）**：QuantPilot 的闭环应使用如下“单向派生约束”：

- `EventLog` 是事实源（不可被其他工件反向修正）。  
- `TradeLedger` 只能由 `EventLog` 投影生成（并可持久化）。  
- `EquityCurve` 只能由 `TradeLedger` + `MarketDataSnapshot` + `ExecutionAssumptionSpec.valuation_policy` 确定性生成。  
- `Metrics` 只能由 `EquityCurve`（returns 序列）+ `TradeLedger`（交易统计）生成。  

这能保证：  
- 审计时可以沿着链条逐层解释；  
- 比较实验时可以定位差异发生在“事件层（成交不同）”还是“估值口径层（权益不同）”；  
- 可恢复时可从事实源重建投影。

### 关键一致性约束（建议写进协议）

**我的判断（QuantPilot 推荐）**：把下面这些约束写成协议规则（甚至在 CI 中做校验），它们是“研究级”的分水岭：

- **单调序号与稳定排序**：EventLog 的 `seq` 必须严格单调；同一 `ts_event` 的并发事件用 `seq` 定义确定顺序（避免非确定性）。  
- **确定的时间语义**：统一 UTC；区分 `ts_event` 与 `ts_observed`（借鉴 OpenTelemetry 的 Timestamp 与 ObservedTimestamp 的区分）。citeturn3search15turn3search3  
- **订单生命周期可串联**：至少提供 `client_order_id` 与 `order_id`，以及每个 fill 的 `fill_id`，与行业标准对“订单/回报 ID”的区分一致。citeturn1search3turn1search27  
- **一处定义成本口径**：费用与滑点必须在 ExecutionAssumptionSpec 定义，Ledger/Equity/Metrics 只引用与展开，不得各自“重新假设”。  
- **投影可校验**：TradeLedger 的每条 fill 必须能回链到 EventLog 的某条 Fill 事件；EquityCurve 的每个采样点必须能回链到该时刻之前 ledger 的状态（可选：存 position snapshot hash）。

### 借鉴 ML 元数据图的“节点/边”表达

**事实（来自资料）**：Vertex ML Metadata 用图结构表示 lineage：artifacts 与 executions 为节点，events 连接输入/输出。citeturn5search22  

**我的判断（QuantPilot 推荐）**：即使 QuantPilot 暂时不引入图数据库，也应在 Manifest 中显式记录边关系（inputs/outputs/derived_from），这样未来迁移到更强 lineage 系统（或对接 OpenLineage）不会推倒重来。OpenLineage 也使用“Run/Job/Dataset + facets”的可扩展模型，特别适合把新信息（如 exchange 规则、数据质量统计）以 facet 的形式添加，而不破坏主结构。citeturn5search4turn5search1turn5search7  

## 版本管理、哈希策略与兼容性策略

### Schema versioning 约定

**事实（来自资料）**：在可演化协议中，保持前后兼容的核心是“新增字段可被旧读者忽略、删除字段需要谨慎”；例如 Protocol Buffers 明确提到旧代码会忽略新字段，同时强调删除字段后应把字段号加入 reserved，避免复用导致严重问题。citeturn2search4turn2search0  
**事实（来自资料）**：数据系统常用 BACKWARD/FORWARD/FULL 等兼容性定义来约束 schema 演进策略（Confluent 对兼容性模式有清晰定义）。citeturn1search1  

**我的判断（QuantPilot 推荐）**：QuantPilot 协议采用“双版本”：

- `protocol_version`：QuantPilot backtest 协议大版本（跨 artifact 的通用规则、目录结构、哈希/manifest 约定）。  
- `schema_version`：每个 artifact/spec 自己的 schema 版本（语义版本 SemVer：MAJOR 为破坏性变更，MINOR 为新增字段，PATCH 为修 bug/文档）。  

演进规则（beta 即应执行）：

- MAJOR 变更必须提供迁移器（至少能读旧、写新）。  
- MINOR 只允许新增 optional 字段；旧实现遇到未知字段必须“保留并忽略”，不要 hard fail（对 JSON/Parquet 读者同样如此）。这与 Protobuf“unknown fields”理念一致。citeturn2search4turn2search0  
- 删除字段：先标记 deprecated，至少保留 2 个 MINOR；再在下个 MAJOR 删除，并在 schema 中记录 tombstone（类似 reserved 的思想）。citeturn2search0  

### 哈希策略：bytes hash 与“等价性 hash”

**事实（来自资料）**：内容寻址（content-addressable）思想在 Git 等系统中被广泛使用，对象以内容哈希标识；Nix 也提供内容寻址 store 对象。citeturn2search1turn2search7  

**我的判断（QuantPilot 推荐）**：  
- 对 JSON 类 Spec/Meta：使用 RFC 8785 JCS 规范化后做 sha256，得到稳定 digest。citeturn5search0  
- 对 Parquet/Arrow 等二进制表：  
  - beta 阶段先用 `bytes_hash`（文件字节 sha256）即可；  
  - 研究级增强阶段再引入 `logical_hash`（对表的 canonical row order + canonical encoding 做哈希），用于“同内容不同压缩参数”的等价性判断。  

### “Run 等价性键”（用于比较与缓存）

**事实（来自资料）**：Bazel 的 Remote Execution API 将 Action 视为“复现所需信息的集合”，并可用其编码 digest 作为唯一标识，支持缓存复用。citeturn2search18turn2search14  

**我的判断（QuantPilot 推荐）**：引入 `run_fingerprint`：

```
run_fingerprint = sha256( canonical_json(RunSpec without timestamps) )
```

并规定：  
- `created_at` 等非决定性字段不参与 fingerprint；  
- fingerprint 必须间接覆盖所有输入 digest（通过 RunSpec.inputs 引用）。  

这使得“可重复运行/可比较实验”有一个稳定的键：同 fingerprint 的 run 应产出同 EventLog（或至少同 Ledger/Equity/Metrics），否则说明引擎非确定或存在隐式输入。

## 最小实现版本与研究级增强版的路线图，以及 Axum/前端/目录映射

### 最小实现版本（单机 beta 现在就该做）

目标：在 BTCUSDT + paper 的范围内，把“可复现闭环”做实，且不被未来复杂性拖垮。

**现在就该做（强制）**  
1) **RunSpec/BacktestSpec/ExecutionAssumptionSpec/MarketDataSnapshotSpec 全量落盘 + digest**：没有这一层，就无法做到“同输入可重放”。citeturn4view0turn5search0  
2) **EventLogArtifact 作为事实源**：哪怕事件类型很少，也要先把 append-only、seq、时间语义、订单关联 ID 立规矩。citeturn4view3turn1search3turn3search15  
3) **TradeLedger + EquityCurve + Metrics 默认产出并持久化**：它们是最常用的分析/展示投影，且可以反向校验 EventLog 的一致性。citeturn3search1turn4view3  
4) **ReproducibilityManifest（哈希图）**：把“可解释差异”落到机器可对比的 digest 列表（data/compile/assumption）。citeturn2search1turn2search7turn5search0  
5) **目录结构与 API 以 artifact 为中心设计**：避免未来变成“只剩 UI JSON”。

**现在可以延后（但要预留字段/接口）**  
- 复杂的微观结构（orderbook、partial fill 深度模拟）。  
- 分布式存储、远端 CAS 去重、跨机器共享。  
- 图数据库 lineage（先用 manifest 边关系即可）。citeturn5search22turn5search4  

### 研究级增强版（未来）

在不改变核心协议哲学（EventLog 为事实源 + 投影派生）前提下，增强点建议是“可比较、可解释、可扩展”：

- **CAS 去重与内容寻址对象库**：把大文件（数据快照、事件日志、ledger）放入 `cas/sha256/..`，run 目录只存引用（借鉴 Git/DVC 的内容寻址与元数据分离理念）。citeturn2search1turn4view2  
- **Schema Registry / 兼容性测试**：引入类似 BACKWARD/FORWARD/FULL 的兼容性 gate，在 CI 中阻止破坏性变更（借鉴 Confluent 的兼容性模型）。citeturn1search1  
- **Facet 扩展机制**：参考 OpenLineage facets，把可选域信息（数据质量、撮合细节、策略内部统计）以命名空间 facet 挂载，降低主 schema 的膨胀风险。citeturn5search7turn5search4  
- **可解释差异（diff artifacts）**：对比两个 run 时自动生成 `DiffReportArtifact`：列出输入 digest 差异（data/assumption/compile），并定位到输出差异层级（事件差异 vs 估值差异）。  
- **确定性强化**：资金与价格采用定点数（decimal128 或 integer scaled），并把舍入规则写入 ExecutionAssumptionSpec（避免浮点细微差导致权益曲线漂移）。

### 与 Axum API、前端回测详情页、后端持久化目录结构的映射建议

#### API 设计映射（面向 Axum）

**我的判断（QuantPilot 推荐）**：API 应围绕“Run + Artifacts”组织（与 MLflow/W&B 的 run→artifacts 概念一致）。citeturn4view0turn4view1  

建议最小 API（REST）：

- `POST /api/v1/runs`：提交 RunSpec（或提交 BacktestSpec + refs，由服务端生成 RunSpec），返回 `run_id` 与 `run_fingerprint`。  
- `GET /api/v1/runs/{run_id}`：返回 run 元数据与状态（queued/running/finished/failed），以及关键输入 digest（用于对比）。  
- `GET /api/v1/runs/{run_id}/artifacts`：列出 artifacts（type、schema_version、digest、size、下载地址）。  
- `GET /api/v1/runs/{run_id}/artifacts/{artifact_type}`：获取某类 artifact 的 meta；大 payload 走单独下载/流式接口。  
- `GET /api/v1/runs/{run_id}/equity`：为前端优化的读取接口（底层读取 EquityCurveArtifact）。  
- `GET /api/v1/runs/{run_id}/trades`：读取 TradeLedgerArtifact（支持分页/过滤）。  
- `GET /api/v1/runs/{run_id}/events`：读取 EventLogArtifact（支持按时间/seq 范围分页）。  

#### 前端“回测详情页”与工件的最小映射

**我的判断（QuantPilot 推荐）**：详情页不应直接依赖“某个聚合 JSON”，而应由工件驱动：

- 顶部摘要卡片：MetricsArtifact.summary + ExecutionAssumptionSpec 摘要（费率/滑点/延迟 fingerprint）。  
- 权益曲线图：EquityCurveArtifact（必要输出）。  
- 交易列表：TradeLedgerArtifact（fills + orders 视图）。  
- 日志/事件回放：EventLogArtifact（可以先只展示订单/成交/错误事件，逐步扩展）。  
- “可复现”面板：ReproducibilityManifest（输入 digest 列表、引擎版本、seed、数据快照信息）。  

#### 后端持久化目录结构（单机 beta 方案）

**借鉴（可迁移原则）**：MLflow 默认也可落到本地文件系统进行追踪，说明“单机文件目录 = 最小可行 artifact store”是合理起点。citeturn4view0  

**我的判断（QuantPilot 推荐）**：先采用“run 目录为主、内含 inputs/outputs/manifest”的结构；未来再引入 CAS 去重而不破坏 run 目录语义。

建议目录：

```
qp_store/
  runs/
    run_01J.../
      run_spec.json
      inputs/
        backtest_spec.json
        execution_assumption.json
        market_data_snapshot.json
        strategy/
          strategy_meta.json
          source_bundle.tgz
        compile/
          compile_meta.json
          binary.wasm
        core_ir/
          core_ir_meta.json
          core_ir.json
      outputs/
        event_log/
          artifact_meta.json
          events.parquet
        ledger/
          artifact_meta.json
          fills.parquet
          orders.parquet
          positions.parquet
        equity/
          artifact_meta.json
          equity.parquet
        metrics/
          artifact_meta.json
          metrics.json
          returns.parquet
      manifest.json
      index.json   # 可选：冗余索引，便于列表页快速加载
  schemas/
    1.0.0/
      BacktestSpec.schema.json
      ExecutionAssumptionSpec.schema.json
      ...
```

`index.json`（可选）只存“可快速展示的字段（指标摘要、关键假设摘要、输入 digest）”，避免每次列表页扫描全部大工件。

---

**总结性的取舍（把话说死，避免空泛）**  
- 研究级体系的“第一性原理”在 QuantPilot 这里不是 UI，而是：**用规范化 + 哈希把输入钉死，用事件日志把事实钉死，用 manifest 把差异解释钉死**。事件溯源的审计/回放价值与复杂性警告也意味着：你们应当在 beta 阶段采用“轻量事件溯源内核”（单 run 内 append-only event log + 投影），而不要一开始引入分布式事件总线或全域 CQRS。citeturn4view3turn0search3  
- 只要你们把本文所述的“必要输入/必要输出/manifest + 版本兼容规则”在单机 beta 落地，就已经具备从“基础 replay backtest”演化为“研究级 backtest 体系”的最关键骨架；未来的增强（CAS、lineage 图、schema registry、facet 扩展）都可以在不推倒核心协议的前提下渐进式加入。citeturn2search1turn5search4turn1search1
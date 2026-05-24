/// v3.7.0: 执行端状态管理
use crate::audit_log::AuditLog;
use crate::live_runner::RunnerPool;
use crate::ws_client::WsEvent;
use qrpc_core::{CoreStrategyIr, Symbol};
use qrpc_core_ir::{
    CoreMetadata, CoreSourceKind, CoreTimeInForce, ExecutionRule, ExecutionSizingKind,
};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;

#[derive(Debug, Clone, serde::Serialize)]
pub struct KlineBar {
    pub open_time_ms: u64,
    pub close_time_ms: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub struct RingBuffer {
    pub bars: VecDeque<KlineBar>,
    pub capacity: usize,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            bars: VecDeque::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }
    pub fn push(&mut self, bar: KlineBar) {
        if self.bars.len() >= self.capacity {
            self.bars.pop_front();
        }
        self.bars.push_back(bar);
    }
    pub fn latest(&self) -> Option<&KlineBar> {
        self.bars.back()
    }
}

#[derive(Debug, Clone)]
pub struct ActiveStrategy {
    pub strategy_id: String,
    pub name: String,
    pub core_ir: CoreStrategyIr,
    pub graph_json: serde_json::Value,
    pub params: BTreeMap<String, serde_json::Value>,
    pub status: StrategyStatus,
    pub subscribed_symbols: Vec<Symbol>,
    pub execution_mode: ExecutionMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyStatus {
    Loaded,
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecutionMode {
    Paper,
    Live,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::Paper
    }
}

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub strategy_id: String,
    pub trigger_type: String,
    pub node_id: String,
    pub strength: f64,
    pub occurred_at_ms: u64,
}

pub struct ExecutorState {
    pub strategies: RwLock<BTreeMap<String, ActiveStrategy>>,
    pub kline_buffers: RwLock<HashMap<String, RingBuffer>>, // v3.2.0: O(1)查找
    pub trigger_events: RwLock<Vec<TriggerEvent>>,
    pub pending_params: RwLock<HashMap<String, BTreeMap<String, serde_json::Value>>>,
    pub params_snapshots: RwLock<BTreeMap<String, Vec<BTreeMap<String, serde_json::Value>>>>,
    /// v3.7.0: RunnerPool (Arc<Mutex<>> 共享, 事件循环 + API handlers 均可访问)
    pub runner_pool: Mutex<Option<Arc<Mutex<RunnerPool>>>>,
    /// WS事件发送端: exchange_name -> tx
    pub ws_tx_map: RwLock<BTreeMap<String, mpsc::UnboundedSender<WsEvent>>>,
    /// v3.5.0: 全局执行模式 (Paper/Live), 影响WS连接和下单路径
    pub global_mode: RwLock<ExecutionMode>,
    pub audit_log: AuditLog,
}

impl ExecutorState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            strategies: RwLock::new(BTreeMap::new()),
            kline_buffers: RwLock::new(HashMap::new()),
            trigger_events: RwLock::new(Vec::new()),
            pending_params: RwLock::new(HashMap::new()),
            params_snapshots: RwLock::new(BTreeMap::new()),
            runner_pool: Mutex::new(None),
            ws_tx_map: RwLock::new(BTreeMap::new()),
            global_mode: RwLock::new(ExecutionMode::Paper),
            audit_log: AuditLog::new(&default_storage_dir()),
        })
    }

    pub fn load_default_or_new() -> Arc<Self> {
        Self::load_state(&default_state_path()).unwrap_or_else(Self::new)
    }

    /// v3.5.0: 读取当前全局执行模式
    pub fn current_mode(&self) -> ExecutionMode {
        self.global_mode
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// v3.5.0: 设置全局执行模式, 返回旧模式
    pub fn set_mode(&self, mode: ExecutionMode) -> ExecutionMode {
        let mut current = self.global_mode.write().unwrap_or_else(|e| e.into_inner());
        let old = current.clone();
        *current = mode;
        old
    }

    const MAX_STRATEGIES: usize = 50;

    pub fn register(&self, s: ActiveStrategy) -> anyhow::Result<()> {
        let mut strategies = self
            .strategies
            .write()
            .map_err(|e| anyhow::anyhow!("锁: {}", e))?;
        // v3.0.2 B-1: 策略数上限保护
        if strategies.len() >= Self::MAX_STRATEGIES && !strategies.contains_key(&s.strategy_id) {
            anyhow::bail!(
                "策略数已达上限 ({}), 请先停止或删除旧策略",
                Self::MAX_STRATEGIES
            );
        }
        strategies.insert(s.strategy_id.clone(), s);
        drop(strategies);
        // v3.7.0: 注册成功后持久化状态
        self.persist()?;
        Ok(())
    }

    /// v3.7.0 S3: 持久化策略状态到 storage/.executor-state.json
    pub fn persist(&self) -> anyhow::Result<()> {
        let strategies = self
            .strategies
            .read()
            .map_err(|e| anyhow::anyhow!("锁: {}", e))?;
        let simplified: BTreeMap<String, serde_json::Value> = strategies
            .iter()
            .map(|(id, s)| {
                (
                    id.clone(),
                    serde_json::json!({
                        "strategy_id": s.strategy_id,
                        "name": s.name,
                        "params": s.params,
                    }),
                )
            })
            .collect();
        let json = serde_json::to_string_pretty(&simplified)?;
        let path = default_state_path();
        write_file_atomically(&path, json.as_bytes())?;
        Ok(())
    }

    /// v3.7.0 S3: 持久化策略状态到指定路径，供测试与迁移门禁复用。
    #[cfg(test)]
    pub fn persist_to_path(&self, path: &Path) -> anyhow::Result<()> {
        let strategies = self
            .strategies
            .read()
            .map_err(|e| anyhow::anyhow!("锁: {}", e))?;
        let simplified: BTreeMap<String, serde_json::Value> = strategies
            .iter()
            .map(|(id, s)| {
                (
                    id.clone(),
                    serde_json::json!({
                        "strategy_id": s.strategy_id,
                        "name": s.name,
                        "params": s.params,
                    }),
                )
            })
            .collect();
        let json = serde_json::to_string_pretty(&simplified)?;
        write_file_atomically(path, json.as_bytes())?;
        Ok(())
    }

    /// v3.7.0 S3: 从持久化文件加载策略状态
    pub fn load_from_file(path: &Path) -> anyhow::Result<Arc<Self>> {
        let content = std::fs::read_to_string(path)?;
        let data: BTreeMap<String, serde_json::Value> = serde_json::from_str(&content)?;
        let mut strategies = BTreeMap::new();
        for (id, val) in data {
            let name = val
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let params = val
                .get("params")
                .and_then(|v| v.as_object())
                .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                .unwrap_or_default();
            strategies.insert(
                id.clone(),
                ActiveStrategy {
                    strategy_id: id.clone(),
                    name,
                    core_ir: CoreStrategyIr::new(
                        CoreMetadata {
                            strategy_id: id.clone(),
                            name: String::new(),
                            source_kind: CoreSourceKind::RuntimeProtocol,
                        },
                        ExecutionRule {
                            execution_id: String::new(),
                            venue_kind: "paper".into(),
                            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
                            slippage_bps: 0.0,
                            taker_fee_bps: 0.0,
                            total_cost_buffer_bps: 0.0,
                            time_in_force: CoreTimeInForce::Gtc,
                            params: BTreeMap::new(),
                        },
                    ),
                    graph_json: serde_json::Value::Null,
                    params,
                    status: StrategyStatus::Loaded,
                    subscribed_symbols: vec![],
                    execution_mode: ExecutionMode::Paper,
                },
            );
        }
        Ok(Arc::new(Self {
            strategies: RwLock::new(strategies),
            kline_buffers: RwLock::new(HashMap::new()),
            trigger_events: RwLock::new(Vec::new()),
            pending_params: RwLock::new(HashMap::new()),
            params_snapshots: RwLock::new(BTreeMap::new()),
            runner_pool: Mutex::new(None),
            ws_tx_map: RwLock::new(BTreeMap::new()),
            global_mode: RwLock::new(ExecutionMode::Paper),
            audit_log: AuditLog::new(path.parent().unwrap_or_else(|| Path::new("storage"))),
        }))
    }

    /// v3.7.0 S3: 尝试从磁盘恢复状态, 不存在则返回 None
    pub fn load_state(path: &Path) -> Option<Arc<Self>> {
        if !path.exists() {
            return None;
        }
        Self::load_from_file(path).ok()
    }
}

fn default_storage_dir() -> PathBuf {
    std::env::var_os("QUANTPILOT_STORAGE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("storage"))
}

fn default_state_path() -> PathBuf {
    default_storage_dir().join(".executor-state.json")
}

fn write_file_atomically(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("状态文件缺少父目录: {}", path.display()))?;
    std::fs::create_dir_all(dir)?;
    let tmp_path = path.with_extension(format!("tmp-{}", std::process::id()));
    let bak_path = path.with_extension("bak");

    {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&tmp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }

    if path.exists() {
        std::fs::copy(path, &bak_path)?;
    }

    if let Err(rename_error) = std::fs::rename(&tmp_path, path) {
        if bak_path.exists() {
            let _ = std::fs::copy(&bak_path, path);
        }
        let _ = std::fs::remove_file(&tmp_path);
        anyhow::bail!("状态文件原子替换失败: {}", rename_error);
    }

    let _ = std::fs::remove_file(&bak_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_capacity_enforced() {
        let mut buf = RingBuffer::new(3);
        for i in 1..=4 {
            buf.push(KlineBar {
                open_time_ms: i,
                close_time_ms: i + 1,
                open: i as f64,
                high: (i + 1) as f64,
                low: i as f64,
                close: (i as f64) + 0.5,
                volume: 10.0,
            });
        }
        assert_eq!(buf.bars.len(), 3);
        assert_eq!(buf.bars[0].open_time_ms, 2);
    }

    #[test]
    fn ring_buffer_zero_capacity_clamped() {
        let buf = RingBuffer::new(0);
        assert_eq!(buf.capacity, 1);
    }

    #[test]
    fn ring_buffer_capacity_one() {
        let mut buf = RingBuffer::new(1);
        buf.push(KlineBar {
            open_time_ms: 1,
            close_time_ms: 2,
            open: 1.0,
            high: 2.0,
            low: 1.0,
            close: 1.5,
            volume: 10.0,
        });
        assert_eq!(buf.bars.len(), 1);
        buf.push(KlineBar {
            open_time_ms: 2,
            close_time_ms: 3,
            open: 2.0,
            high: 3.0,
            low: 2.0,
            close: 2.5,
            volume: 10.0,
        });
        assert_eq!(buf.bars.len(), 1);
        assert_eq!(buf.bars[0].open_time_ms, 2);
    }

    #[test]
    fn ring_buffer_empty_latest_returns_none() {
        let buf = RingBuffer::new(5);
        assert!(buf.latest().is_none());
    }

    #[test]
    fn persist_to_path_writes_state_without_tmp_or_bak_leftovers() {
        let dir = std::env::temp_dir().join(format!(
            "quantpilot_executor_state_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".executor-state.json");
        let state = ExecutorState::new();

        state.persist_to_path(&path).unwrap();

        assert!(path.exists());
        assert!(!path
            .with_extension(format!("tmp-{}", std::process::id()))
            .exists());
        assert!(!path.with_extension("bak").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}

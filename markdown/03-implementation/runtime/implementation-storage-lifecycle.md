# 存储生命周期设计

> 依据 General_Policy §7 | 生效日期: 2026-05-05

---

## 一、三级分类映射

### 长期 (Permanent)

| 目录 | 写入触发 | 清理方式 |
|------|---------|---------|
| `storage/graphs/` | 图保存 / QS 编译保存 | DELETE API |
| `storage/audit/` | 图保存/删除/回测/运行创建 | 手动归档（未来） |

### 暂时 (Temporary) — TTL 7 天

| 目录 | 写入触发 | 当前状态 |
|------|---------|:--:|
| `storage/backtests/` (非 transient) | 回测保存 API | ❌ 无清理 |
| `storage/runs/` | 运行保存 API | ❌ 无清理 |
| `storage/experiments/` | 实验保存 API | ❌ 无清理 |
| `storage/approvals/` | AI 提案审批 | ⚠️ 状态过期但文件不删 |
| `storage/reports/` (持久化记录) | 证据报告生成 | ⚠️ temp 有 TTL，持久化记录无 |
| `storage/mutations/` | 参数突变提案 | ❌ 无清理 |

### 瞬间 (Transient) — TTL 1 小时

| 目录 | 写入触发 | 当前状态 |
|------|---------|:--:|
| `storage/test-runs/` | `@save_run` TestAction | ❌ 无清理 |
| `storage/ai-proposals/` | AI 提案创建/状态变更 | ❌ 无清理 |
| `storage/sandbox-reports/` | 沙箱验证请求 | ❌ 无清理 |
| `storage/snapshots/` | 部署签名快照 | ❌ 无清理 |
| `storage/alerts/` | 告警触发/确认 | ❌ 无清理 |
| `storage/chaos/` | 混沌实验报告 | ⚠️ temp_pressure 清理了 |
| `storage/reports/` (临时生成) | 报告生成中 | ✅ 已有 24h TTL |

---

## 二、现有带 TTL 机制（保留并统一）

| 机制 | 文件 | 当前 TTL |
|------|------|:--:|
| `PROMOTION_WORK_DIR_TTL_MS` | `backtest_artifacts.rs:36` | 24h |
| `TRANSIENT_BACKTEST_TTL_MS` | `backtest_artifacts.rs:41` | 24h |
| `RUNTIME_REPORT_TRANSIENT_OUTPUT_TTL_MS` | `runtime_persistence.rs:45` | 24h |
| 启动清理 `.saving-*` / `.replacing-*` | `main.rs:629` | — |
| 启动清理 `transient-backtest-*` | `main.rs:642` | — |
| `PROMOTION_WORK_DIR_MAX_*` (32/512MB/256MB) | `backtest_artifacts.rs:37-39` | 配额 |
| `TRANSIENT_BACKTEST_MAX_*` (32/512MB/256MB) | `backtest_artifacts.rs:42-44` | 配额 |

---

## 三、实现方案

### 3-1 新增 `storage_lifecycle.rs` 模块

```rust
// src/storage_lifecycle.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageLifecycle {
    Permanent,   // 长期
    Temporary,   // 暂时 (7d)
    Transient,   // 瞬间 (1h)
}

impl StorageLifecycle {
    pub fn ttl(&self) -> Option<std::time::Duration> {
        let dev_mode = std::env::var("QUANTPILOT_DEV").unwrap_or_default() == "true";
        match self {
            Self::Permanent => None,
            Self::Temporary => Some(std::time::Duration::from_secs(
                if dev_mode { 24 * 3600 } else { 7 * 24 * 3600 }
            )),
            Self::Transient => Some(std::time::Duration::from_secs(
                if dev_mode { 10 * 60 } else { 3600 }
            )),
        }
    }
}

const GLOBAL_MAX_BYTES: u64 = 500 * 1024 * 1024; // 500 MB
const WARN_AT_BYTES: u64 = 400 * 1024 * 1024; // 80%
const FORCE_CLEAN_AT_BYTES: u64 = 450 * 1024 * 1024; // 90%

const TEMPORARY_DIR_MAX_BYTES: u64 = 200 * 1024 * 1024;
const TRANSIENT_DIR_MAX_BYTES: u64 = 50 * 1024 * 1024;

// 目录 → 生命周期映射
fn directory_lifecycle(dir_name: &str) -> StorageLifecycle {
    match dir_name {
        "graphs" | "audit" => StorageLifecycle::Permanent,
        "backtests" | "runs" | "experiments" | "approvals" 
        | "reports" | "mutations" => StorageLifecycle::Temporary,
        _ => StorageLifecycle::Transient,
        // test-runs, ai-proposals, sandbox-reports, snapshots, alerts, chaos
    }
}
```

### 3-2 启动清理函数

```rust
pub fn startup_storage_cleanup(storage_root: &Path) {
    let entries = match std::fs::read_dir(storage_root) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut total_size: u64 = 0;
    let mut cleaned_count = 0;
    let now = std::time::SystemTime::now();

    for entry in entries.flatten() {
        let path = entry.path();
        let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
        let lifecycle = directory_lifecycle(&dir_name);
        
        let dir_size = dir_size_bytes(&path);
        total_size += dir_size;

        if let Some(ttl) = lifecycle.ttl() {
            // 遍历目录中的文件，删除超过 TTL 的
            if let Ok(files) = std::fs::read_dir(&path) {
                for file in files.flatten() {
                    if let Ok(meta) = file.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if now.duration_since(modified).unwrap_or_default() > ttl {
                                let _ = std::fs::remove_file(file.path());
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    if total_size > WARN_AT_BYTES {
        eprintln!("[storage] WARNING: total size {} MB exceeds 80% threshold ({} MB)",
            total_size / 1024 / 1024, WARN_AT_BYTES / 1024 / 1024);
    }
    if cleaned_count > 0 {
        eprintln!("[storage] startup cleanup: removed {} expired files", cleaned_count);
    }
}
```

### 3-3 定时清理（每 1 小时）

在 `main.rs` 的后台循环中新增：

```rust
// 每小时清理一次过期暂时/瞬间数据
if tick_count % 60 == 0 {
    storage_lifecycle::startup_storage_cleanup(&state.graph_store_dir.parent().unwrap());
}
```

### 3-4 test-runs 的立即清理

`test_runner.rs` 中的 `SaveRun` 已经将文件写入 `storage/test-runs/`。应该在测试场景结束后立即清理：

```rust
// 在 TestRunner::execute() 返回前
if dev_mode {
    let test_runs_dir = Path::new("storage").join("test-runs");
    if test_runs_dir.exists() {
        // 保留最近 5 个，删除其余
        cleanup_oldest(&test_runs_dir, keep_count: 5);
    }
}
```

---

## 四、各目录处理策略

| 目录 | 当前 | 目标 | 实施 |
|------|:--:|:--:|------|
| `graphs/` | ✅ | 长期 | 不变 |
| `audit/` | ✅ | 长期 | 不变 |
| `backtests/` | ❌ | 暂时 7d | 启动清理扫描 >7d 文件删除 |
| `runs/` | ❌ | 暂时 7d | 启动清理 + 保留最近 10 个 |
| `experiments/` | ❌ | 暂时 7d | 启动清理 |
| `approvals/` | ⚠️ | 暂时 7d | 启动清理 |
| `reports/` | ⚠️ | 暂时 7d (持久化) + 瞬间 1h (临时) | 统一 TTL |
| `mutations/` | ❌ | 暂时 7d | 启动清理 |
| `test-runs/` | ❌ | 瞬间 1h | 启动清理 + 测试结束即删 |
| `ai-proposals/` | ❌ | 瞬间 1h | 启动清理 |
| `sandbox-reports/` | ❌ | 瞬间 1h | 启动清理 |
| `snapshots/` | ❌ | 瞬间 1h | 启动清理 |
| `alerts/` | ❌ | 瞬间 1h | 启动清理（告警已确认后） |
| `chaos/` | ⚠️ | 瞬间 1h | 启动清理 |

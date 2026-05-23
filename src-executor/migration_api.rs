use crate::executor_state::{ActiveStrategy, ExecutionMode, ExecutorState, StrategyStatus};
/// v3.7.0: 策略迁移 API — 接收测试端部署的策略包
/// 验证签名 → 解密 → 反序列化 → 注册到执行端
use anyhow::{bail, Result};
use qrpc_core::CoreStrategyIr;
use std::collections::BTreeMap;
use std::sync::Arc;

/// 策略迁移包 (从测试端接收) — v3.1.0 D-1: deny_unknown_fields
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyPackage {
    pub strategy_id: String,
    pub name: String,
    pub core_ir: CoreStrategyIr,
    pub graph_json: serde_json::Value,
    pub params_snapshot: BTreeMap<String, serde_json::Value>,
    pub compile_id: String,
    pub migrated_at_ms: u64,
    pub signature: String,
    /// v3.0.0 A-1: QS管道溯源证明 — 证明 core_ir 经过完整QS编译链
    #[serde(default)]
    pub qs_proof: Option<QsCompileProof>,
    /// v3.7.0 S5: 执行模式, 默认 Paper
    #[serde(default)]
    pub execution_mode: ExecutionMode,
}

/// v3.7.0: QS编译溯源证明 — 验证策略经过完整QS管道编译
#[derive(Debug, serde::Deserialize)]
pub struct QsCompileProof {
    pub qs_source_hash: String,
    pub protocol_hash: String,
}

/// 验证策略包 SHA-256 签名 + QS管道溯源
pub fn verify_package_signature(pkg: &StrategyPackage) -> Result<()> {
    // v3.0.0 A-1: QS溯源验证 — 拒绝无QS证明的策略包
    match &pkg.qs_proof {
        Some(proof) => {
            if proof.qs_source_hash.is_empty() || proof.protocol_hash.is_empty() {
                bail!("QS编译溯源证明不完整: qs_source_hash 和 protocol_hash 不能为空");
            }
        }
        None => {
            bail!("策略包缺少QS编译溯源证明: 请通过测试端 /api/executor/deploy 部署，该端点自动附加QS管道溯源信息");
        }
    }

    // 构建签名输入 (不含 signature 字段本身) — v3.2.0 S0修复: 与发送端字段对齐
    let qs_source_hash = pkg
        .qs_proof
        .as_ref()
        .map(|p| p.qs_source_hash.clone())
        .unwrap_or_default();
    let sig_input = serde_json::json!({
        "strategy_id": pkg.strategy_id,
        "name": pkg.name,
        "compile_id": pkg.compile_id,
        "migrated_at_ms": pkg.migrated_at_ms,
        "core_ir_hash": qrpc_core::canonical_json_sha256_digest(
            &serde_json::to_value(&pkg.core_ir).unwrap_or_default()
        ).map(|d| d.value).unwrap_or_default(),
        "qs_source_hash": qs_source_hash,
        "graph_json_hash": qrpc_core::canonical_json_sha256_digest(&pkg.graph_json).map(|d| d.value).unwrap_or_default(),
        "params_snapshot_hash": qrpc_core::canonical_json_sha256_digest(&serde_json::to_value(&pkg.params_snapshot).unwrap_or_default()).map(|d| d.value).unwrap_or_default(),
    });
    let expected = qrpc_core::canonical_json_sha256_digest(&sig_input)
        .map(|d| d.value)
        .map_err(|e| anyhow::anyhow!("签名计算失败: {}", e))?;
    if expected != pkg.signature {
        bail!("策略包签名验证失败: 可能被篡改或传输错误");
    }
    Ok(())
}

/// 解密并加载策略包 (Phase 1: 明文, Phase 2+加密通道后启用)
pub fn decrypt_package(encrypted: &[u8]) -> Result<StrategyPackage> {
    // Phase 1: 直接 JSON 反序列化 (执行端和测试端在 localhost 同一进程)
    // Phase 2: 使用 session_crypto::decrypt 解密后反序列化
    let pkg: StrategyPackage = serde_json::from_slice(encrypted)?;
    Ok(pkg)
}

/// 加载策略到执行端状态
pub fn load_strategy(state: &Arc<ExecutorState>, pkg: StrategyPackage) -> Result<()> {
    verify_package_signature(&pkg)?;

    let symbols: Vec<qrpc_core::Symbol> = pkg
        .core_ir
        .data_bindings
        .iter()
        .filter_map(|d| d.source_hints.get("symbol").cloned())
        .map(|s| qrpc_core::Symbol::Other(s))
        .collect();

    let strategy = ActiveStrategy {
        strategy_id: pkg.strategy_id.clone(),
        name: pkg.name,
        core_ir: pkg.core_ir,
        graph_json: pkg.graph_json,
        params: pkg.params_snapshot,
        status: StrategyStatus::Loaded,
        subscribed_symbols: symbols,
        execution_mode: pkg.execution_mode,
    };

    state.register(strategy)?;
    Ok(())
}

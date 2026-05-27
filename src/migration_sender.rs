/// v3.0.0: 测试端策略迁移发送器
/// 编译策略 → 构建迁移包 → 发送到执行端 (:3001)
use anyhow::{bail, Result};
use axum::Json;
use qrpc_core::CoreStrategyIr;
use serde_json::Value;
use std::collections::BTreeMap;

/// 策略迁移包 — v3.1.0 D-2: deny_unknown_fields
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationPackage {
    pub strategy_id: String,
    pub name: String,
    pub core_ir: CoreStrategyIr,
    pub graph_json: Value,
    pub params_snapshot: BTreeMap<String, Value>,
    pub compile_id: String,
    pub migrated_at_ms: u64,
    pub signature: String,
    /// v3.0.0 A-1: QS管道溯源证明
    pub qs_proof: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy_config_preflight: Option<Value>,
}

/// 测试端 API: 接收前端部署请求, 编译→打包→发送到执行端
pub async fn deploy_strategy(
    Json(body): Json<Value>,
) -> Result<Json<Value>, (axum::http::StatusCode, String)> {
    let graph_json = body.get("graph_json").cloned().ok_or_else(|| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            "缺少 graph_json".to_string(),
        )
    })?;

    // 1. QS lowering → RuntimeProtocolCoreConfig
    let protocol =
        crate::compile_api::compile_runtime_protocol_via_qs(&graph_json).map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                format!("编译失败: {}", e.1),
            )
        })?;

    // 2. 编译 → Core IR
    let compiled = qrpc_compiler::compile_runtime_protocol_config(&protocol).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            format!("Core IR 编译失败: {}", e),
        )
    })?;

    let name = graph_json
        .get("metadata")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("未命名策略");
    let params = BTreeMap::new();
    let graph_id = graph_json
        .get("metadata")
        .and_then(|v| v.get("graph_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let compile_id = format!(
        "{}_{}",
        graph_id,
        compiled.config_hash.chars().take(8).collect::<String>()
    );

    let strategy_config_preflight =
        crate::strategy_config_api::build_strategy_config_preflight_value(
            crate::strategy_config_api::StrategyConfigArtifactRequest {
                strategy_id: Some(graph_id.to_string()),
                strategy_version: Some(compile_id.clone()),
                source_mode: Some("strategy_graph".to_string()),
                graph_json: Some(graph_json.clone()),
                runtime_config: Some(serde_json::to_value(&protocol).unwrap_or_default()),
                qs_source: None,
                core_ir: Some(serde_json::to_value(&compiled.core_ir).unwrap_or_default()),
                v4_graph: None,
                capability_snapshot_hash: None,
                capability_source: None,
                runtime_mode: Some("PaperSimulated".to_string()),
                evidence_anchors: vec![crate::strategy_config_api::EvidenceAnchorInput {
                    anchor_type: "compile".to_string(),
                    anchor_id: Some(compile_id.clone()),
                    digest: Some(compiled.config_hash.clone()),
                    summary: Some("executor deploy compile".to_string()),
                }],
                proposal_bindings: vec![],
                required_execution_capability_sources: vec!["runtime_simulated".to_string()],
            },
        )?;

    let mut pkg =
        build_migration_package(name, &compiled.core_ir, &graph_json, &params, &compile_id)
            .map_err(|e| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("{}", e),
                )
            })?;
    pkg.strategy_config_preflight = Some(strategy_config_preflight);

    let strategy_id = send_to_executor(&pkg).await.map_err(|e| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            format!("{}", e),
        )
    })?;

    Ok(Json(serde_json::json!({
        "status": "deployed",
        "strategy_id": strategy_id,
        "message": "策略已部署到实时执行端",
    })))
}

/// 构建策略迁移包
pub fn build_migration_package(
    strategy_name: &str,
    core_ir: &CoreStrategyIr,
    graph_json: &Value,
    params: &BTreeMap<String, Value>,
    compile_id: &str,
) -> Result<MigrationPackage> {
    let strategy_id = format!("exec-{}", compile_id.chars().take(12).collect::<String>());
    let migrated_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // v3.0.0 A-1: 构建QS管道溯源证明
    let qs_source_hash = qrpc_core::canonical_json_sha256_digest(graph_json)
        .map(|d| d.value)
        .unwrap_or_default();
    let protocol_hash =
        qrpc_core::canonical_json_sha256_digest(&serde_json::to_value(core_ir).unwrap_or_default())
            .map(|d| d.value)
            .unwrap_or_default();

    // v3.3.0: 签名覆盖 graph_json + params_snapshot 哈希
    let graph_hash = qrpc_core::canonical_json_sha256_digest(graph_json)
        .map(|d| d.value)
        .unwrap_or_default();
    let params_hash =
        qrpc_core::canonical_json_sha256_digest(&serde_json::to_value(params).unwrap_or_default())
            .map(|d| d.value)
            .unwrap_or_default();

    let signature = qrpc_core::canonical_json_sha256_digest(&serde_json::json!({
        "strategy_id": strategy_id,
        "name": strategy_name,
        "compile_id": compile_id,
        "migrated_at_ms": migrated_at_ms,
        "core_ir_hash": protocol_hash,
        "qs_source_hash": qs_source_hash,
        "graph_json_hash": graph_hash,
        "params_snapshot_hash": params_hash,
    }))
    .map(|d| d.value)
    .map_err(|e| anyhow::anyhow!("签名计算失败: {}", e))?;

    Ok(MigrationPackage {
        strategy_id,
        name: strategy_name.to_string(),
        core_ir: core_ir.clone(),
        graph_json: graph_json.clone(),
        params_snapshot: params.clone(),
        compile_id: compile_id.to_string(),
        migrated_at_ms,
        signature,
        qs_proof: Some(serde_json::json!({
            "qs_source_hash": qs_source_hash,
            "protocol_hash": protocol_hash,
        })),
        strategy_config_preflight: None,
    })
}

/// 发送策略到执行端 (Phase 5: HMAC签名待加密通道完成后启用)
pub async fn send_to_executor(pkg: &MigrationPackage) -> Result<String> {
    let body = serde_json::to_vec(pkg)?;
    tokio::task::spawn_blocking(move || {
        let executor_url = std::env::var("QUANTPILOT_EXECUTOR_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3001".into());
        let path = "/api/executor/strategies";
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let signature = match qrpc_session::sign_request("POST", path, timestamp_ms, &body) {
            Ok(signature) => signature,
            Err(_) => {
                qrpc_session::load_session_key()
                    .map_err(|e| anyhow::anyhow!("加载执行端会话密钥失败: {}", e))?;
                qrpc_session::sign_request("POST", path, timestamp_ms, &body)
                    .map_err(|e| anyhow::anyhow!("执行端请求签名失败: {}", e))?
            }
        };
        let response = ureq::post(&format!("{}/api/executor/strategies", executor_url))
            .set("Content-Type", "application/json")
            .set("X-Executor-Timestamp", &timestamp_ms.to_string())
            .set("X-Executor-Signature", &signature)
            .send_bytes(&body)
            .map_err(|e| anyhow::anyhow!("连接执行端失败: {}。请确认执行端已启动", e))?;
        let status = response.status();
        let text = response.into_string()?;
        if status != 200 {
            bail!("执行端返回错误 ({}): {}", status, text);
        }
        let result: Value = serde_json::from_str(&text)?;
        Ok(result["strategy_id"].as_str().unwrap_or("?").to_string())
    })
    .await
    .map_err(|e| anyhow::anyhow!("{}", e))?
}

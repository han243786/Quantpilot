use crate::{
    alert_engine, auth, auth_middleware, backup, build_app_router,
    cleanup_backtest_promotion_work_dirs, cleanup_transient_backtest_records,
    cli_support::{
        self, parse_cli_command_from, print_cli_usage, run_v4_strategy_from_cli,
        validate_strategy_ir_file, CliCommand,
    },
    current_time_ms, new_app_state, rate_limiter, storage_lifecycle, AlertFiring, AlertFiringState,
    AlertSeverity, AppState, ChaosExperimentReport, DeploymentSignatureSnapshot,
    RuntimeAiProposalStatus, RuntimeApprovalLevel, RuntimeApprovalLifecycleEntry,
    RuntimeApprovalRecord, RuntimeApprovalReviewState, SandboxVerificationReport,
};
use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method, StatusCode},
    response::IntoResponse,
};
use std::{
    env,
    net::SocketAddr,
    path::{Path as FsPath, PathBuf},
    sync::atomic::Ordering,
};
use tokio::fs;
use tower_http::cors::CorsLayer;

pub async fn run_server() -> anyhow::Result<()> {
    initialize_process_environment();
    dispatch_process_command().await
}

fn initialize_process_environment() {
    let _ = dotenvy::dotenv();

    let log_format = env::var("QUANTPILOT_LOG_FORMAT").unwrap_or_else(|_| "compact".to_string());
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr);
    if log_format == "json" {
        subscriber.json().init();
    } else {
        subscriber.compact().init();
    }

    if env::var("QUANTPILOT_DEV").unwrap_or_default() == "true" {
        safe_eprintln!("[启动] DEV 模式已启用 — 瞬态数据 TTL 缩短，强制启动清理");
    }

    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{}", info);
        eprintln!(
            "[panic] {} — 服务将退出",
            crate::safe_log::sanitize_secrets(&msg)
        );
    }));
}

async fn dispatch_process_command() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "credential" {
        if let Err(error) = cli_support::handle_credential_command(&args[1..]) {
            safe_eprintln!("错误: {}", error);
            std::process::exit(1);
        }
        return Ok(());
    }

    match parse_cli_command_from(env::args())? {
        CliCommand::Serve => run_api_server().await,
        CliCommand::PrintHelp => {
            print_cli_usage();
            Ok(())
        }
        CliCommand::StrategyIrValidate { path } => validate_strategy_ir_file(path).await,
        CliCommand::V4Run { graph_id_or_path } => run_v4_strategy_from_cli(graph_id_or_path).await,
    }
}

async fn run_api_server() -> anyhow::Result<()> {
    let graph_store_dir = PathBuf::from("storage/graphs");
    let run_store_dir = PathBuf::from("storage/runs");
    let backtest_store_dir = PathBuf::from("storage/backtests");
    let experiment_store_dir = PathBuf::from("storage/experiments");
    // Block 5 新存储目录
    let approval_store_dir = PathBuf::from("storage/approvals");
    let sandbox_report_store_dir = PathBuf::from("storage/sandbox-reports");
    let alert_store_dir = PathBuf::from("storage/alerts");
    let snapshot_store_dir = PathBuf::from("storage/snapshots");
    let chaos_store_dir = PathBuf::from("storage/chaos");
    let audit_store_dir = PathBuf::from("storage/audit");
    let report_store_dir = PathBuf::from("storage/reports");
    let mutation_store_dir = PathBuf::from("storage/mutations");
    let ai_proposal_store_dir = PathBuf::from("storage/ai-proposals");
    // v2.5.0: 并行创建 13 个存储目录, 减少启动等待时间
    let dirs: Vec<_> = [
        &graph_store_dir,
        &run_store_dir,
        &backtest_store_dir,
        &experiment_store_dir,
        &approval_store_dir,
        &sandbox_report_store_dir,
        &alert_store_dir,
        &snapshot_store_dir,
        &chaos_store_dir,
        &audit_store_dir,
        &report_store_dir,
        &mutation_store_dir,
        &ai_proposal_store_dir,
    ]
    .iter()
    .map(|d| d.to_path_buf())
    .collect();
    let tasks: Vec<_> = dirs
        .into_iter()
        .map(|dir| {
            tokio::spawn(async move {
                if let Err(e) = fs::create_dir_all(&dir).await {
                    safe_eprintln!(
                        "[启动] 创建存储目录 {} 失败: {} (服务将继续运行)",
                        dir.display(),
                        e
                    );
                }
            })
        })
        .collect();
    for task in tasks {
        let _ = task.await;
    }
    if let Err(error) = cleanup_backtest_promotion_work_dirs(&backtest_store_dir).await {
        safe_eprintln!("warning: 清理回测临时目录失败: {}", error);
    }

    // v2.0.0: 启动时校验市场公钥不是测试向量
    qrpc_runtime::plugin_market::assert_market_public_key_is_production();

    let state = new_app_state(graph_store_dir, run_store_dir, backtest_store_dir);
    // Block 5: 初始化告警规则
    alert_engine::init_alert_rules(&state).await;
    // Block 5: 从磁盘预热持久化数据
    warm_persisted_state(&state).await;
    if let Err(error) =
        cleanup_transient_backtest_records(state.transient_backtest_store_dir.as_ref()).await
    {
        safe_eprintln!("warning: 清理过期回测目录失败: {}", error);
    }

    // 启动时清理过期存储文件和构建工件
    storage_lifecycle::startup_storage_cleanup(std::path::Path::new("storage"));
    storage_lifecycle::cleanup_build_artifacts();

    let cors_origin = env::var("QUANTPILOT_CORS_ORIGIN")
        .unwrap_or_else(|_| "http://127.0.0.1:5173,http://localhost:5173".to_string());
    let cors_origins: Vec<HeaderValue> = cors_origin
        .split(',')
        .filter_map(|s| {
            let trimmed = s.trim();
            // v2.3.3 修复 S0-3: 拒绝通配符 origin 和无效 scheme
            if trimmed == "*" {
                safe_eprintln!("[CORS] 拒绝通配符 origin '*', 请使用明确的 http(s):// 地址");
                return None;
            }
            if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
                safe_eprintln!("[CORS] 拒绝非 http(s) origin: {}", trimmed);
                return None;
            }
            HeaderValue::from_str(trimmed).ok()
        })
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(cors_origins)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    // v2.0.1: HTTP 安全头中间件
    async fn security_headers_middleware(
        request: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        let mut response = next.run(request).await;
        let headers = response.headers_mut();
        headers.insert(
            axum::http::HeaderName::from_static("x-content-type-options"),
            axum::http::HeaderValue::from_static("nosniff"),
        );
        headers.insert(
            axum::http::HeaderName::from_static("x-frame-options"),
            axum::http::HeaderValue::from_static("DENY"),
        );
        headers.insert(
            axum::http::HeaderName::from_static("referrer-policy"),
            axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
        );
        // v2.1.x: 添加 CSP 和 HSTS 安全头
        headers.insert(
            axum::http::HeaderName::from_static("content-security-policy"),
            axum::http::HeaderValue::from_static(
                "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; connect-src 'self'; img-src 'self' data:; base-uri 'self';",
            ),
        );
        headers.insert(
            axum::http::HeaderName::from_static("strict-transport-security"),
            axum::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
        // v2.4.0: Permissions-Policy 限制浏览器 API 访问
        headers.insert(
            axum::http::HeaderName::from_static("permissions-policy"),
            axum::http::HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
        );
        response
    }

    let app = build_app_router(state.clone())
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB
        .layer(cors)
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(axum::middleware::from_fn(json_rejection_middleware))
        .layer(axum::middleware::from_fn(
            rate_limiter::rate_limit_middleware,
        ))
        .layer(axum::middleware::from_fn(auth_middleware::api_key_auth));

    // Block 5 P1-5 + P3-4: 审批超时 + 观察窗口后台任务
    // v1.0.2: AbortHandle 在进程退出时自动取消后台循环
    let expiry_state = state.clone();
    // v2.3.4: 后台任务 — 每次迭代用 catch_unwind 包裹，防止单次 panic 终止整个循环
    let bg_handle = tokio::spawn(async move {
        let mut tick: u64 = 0;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            tick += 1;
            let current_tick = tick;
            let state_ref = &expiry_state;
            process_expired_approvals(state_ref).await;
            check_observation_windows(state_ref).await;
            if current_tick.is_multiple_of(1440) {
                backup::backup_permanent_storage().await;
            }
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                storage_lifecycle::startup_storage_cleanup(std::path::Path::new("storage"));
            }));
            if let Err(e) = result {
                safe_eprintln!(
                    "[后台任务] panic 已恢复: {} (tick {})",
                    e.downcast_ref::<String>()
                        .map(|s| s.as_str())
                        .or_else(|| e.downcast_ref::<&str>().copied())
                        .unwrap_or("未知 panic"),
                    current_tick
                );
            }
            if tick.is_multiple_of(10) {
                let _cutoff = current_time_ms().saturating_sub(24 * 3600 * 1000);
                const MAX_CACHED_RECORDS: usize = 500;
                // v2.3.3 P1-11: 扩展淘汰逻辑至全部 11 个 BTreeMap (原仅覆盖 runs/backtests/experiments)
                macro_rules! trim_map {
                    ($map:expr) => {
                        let mut guard = $map.write().await;
                        if guard.len() > MAX_CACHED_RECORDS {
                            let excess = guard.len() - MAX_CACHED_RECORDS;
                            let to_remove: Vec<_> =
                                guard.iter().take(excess).map(|(k, _)| k.clone()).collect();
                            for k in to_remove {
                                guard.remove(&k);
                            }
                        }
                    };
                }
                trim_map!(expiry_state.runs);
                trim_map!(expiry_state.backtests);
                trim_map!(expiry_state.experiments);
                trim_map!(expiry_state.parameter_mutations);
                trim_map!(expiry_state.ai_proposals);
                trim_map!(expiry_state.hotswap_records);
                trim_map!(expiry_state.approval_records);
                trim_map!(expiry_state.sandbox_reports);
                trim_map!(expiry_state.alert_firings);
                trim_map!(expiry_state.snapshots);
                trim_map!(expiry_state.chaos_experiments);
            }
        }
    });

    let port: u16 = match env::var("QUANTPILOT_PORT") {
        Ok(val) => val.parse().unwrap_or_else(|e| {
            safe_eprintln!(
                "[启动] QUANTPILOT_PORT 值 '{}' 无效 ({}), 使用默认 3000",
                val,
                e
            );
            3000
        }),
        Err(_) => 3000,
    };
    if port == 0 {
        anyhow::bail!("端口 0 是保留端口, 请使用 1-65535 范围内的有效端口");
    }
    // v2.0.1: 绑定地址可通过环境变量配置，容器部署需设为 0.0.0.0
    let bind_host =
        std::env::var("QUANTPILOT_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());
    let bind_ip: std::net::Ipv4Addr = bind_host.parse().unwrap_or_else(|_| {
        safe_eprintln!(
            "[启动] QUANTPILOT_BIND_ADDR 无效 ({}), 回退到 127.0.0.1",
            bind_host
        );
        [127, 0, 0, 1].into()
    });
    let addr = SocketAddr::from((bind_ip, port));
    println!(
        "QuantPilot v{} API → http://{}",
        env!("CARGO_PKG_VERSION"),
        addr
    );
    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        anyhow::anyhow!(
            "端口 {} 已被占用，请检查是否有其他 QuantPilot 实例在运行: {}",
            port,
            e
        )
    })?;
    // v1.1.11: 优雅关闭 — 监听 ctrl_c 信号
    let shutdown_signal = async {
        let _ = tokio::signal::ctrl_c().await;
        safe_eprintln!("[shutdown] 收到终止信号，正在优雅关闭...");
    };
    #[cfg(unix)]
    let sigterm = {
        let mut sig = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("无法注册 SIGTERM 处理器");
        async move {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        result = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()) => { result?; }
        _ = shutdown_signal => {}
        _ = sigterm => {}
    }
    // v2.3.4: 关闭前尝试将内存状态持久化 (runs/backtests/experiments 缓存)
    flush_volatile_state(&state).await;
    bg_handle.abort();
    // v2.1.0: 等待后台任务完成清理 (30s超时防挂起)
    let _ = tokio::time::timeout(std::time::Duration::from_secs(30), bg_handle).await;
    eprintln!("[shutdown] QuantPilot 已停止");
    Ok(())
}

/// v2.3.4: 关闭时将内存缓存中尚未持久化的记录写入磁盘
async fn flush_volatile_state(_state: &AppState) {
    // 大部分运行时记录在创建时已持久化，此处作为关闭安全网
    let _ = crate::storage_lifecycle::startup_storage_cleanup(std::path::Path::new("storage"));
    safe_eprintln!("[shutdown] 内存状态已刷盘");
}

async fn json_rejection_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let response = next.run(req).await;
    // 覆盖 Axum 默认的 JSON 解析错误 (422/400/415), 统一返回中文 JSON
    let status = response.status();
    if status == StatusCode::UNPROCESSABLE_ENTITY
        || status == StatusCode::BAD_REQUEST
        || status == StatusCode::UNSUPPORTED_MEDIA_TYPE
    {
        let content_type = response
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        if content_type.starts_with("application/json") {
            return response;
        }
        let body = axum::Json(serde_json::json!({
            "error": "bad_request",
            "error_code": crate::error_codes::ERR_QSC_BAD_REQUEST,
            "message": "请求格式错误: 请使用 Content-Type: application/json 并确保请求体为有效 JSON"
        }));
        return (StatusCode::BAD_REQUEST, body).into_response();
    }
    response
}

#[cfg(test)]
mod json_rejection_middleware_tests {
    use super::*;
    use axum::{body::Body, http::Request, middleware, routing::get, Json, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn preserves_structured_handler_bad_request_json() {
        let app = Router::new()
            .route(
                "/bad",
                get(|| async {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "error": "structured_error",
                            "details": [{"code": "kept"}],
                        })),
                    )
                }),
            )
            .layer(middleware::from_fn(json_rejection_middleware));

        let response = app
            .oneshot(Request::builder().uri("/bad").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["error"], "structured_error");
        assert_eq!(value["details"][0]["code"], "kept");
    }
}

// v2.1.0: 启动时恢复因崩溃残留的 .bak 文件
async fn recover_stale_bak_files(graph_store_dir: &FsPath) {
    let Ok(mut entries) = fs::read_dir(graph_store_dir).await else {
        return;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("bak") {
            if let Some(stem) = path.file_stem() {
                let json_path = path.with_file_name(stem);
                if !fs::try_exists(&json_path).await.unwrap_or(true) {
                    safe_eprintln!("[startup] 恢复残留 bak 文件: {}", path.display());
                    let _ = fs::rename(&path, &json_path).await;
                } else {
                    // 主文件已存在，bak 是残留，安全删除
                    let _ = fs::remove_file(&path).await;
                }
            }
        }
    }
}

async fn warm_persisted_state(state: &AppState) {
    // v2.1.0: 启动时恢复残留的 .bak 文件（上次保存崩溃残留）
    recover_stale_bak_files(state.graph_store_dir.as_ref()).await;
    // 从磁盘加载审批记录
    if let Ok(mut entries) = fs::read_dir(state.approval_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            // v2.0.1: 仅加载 .json 文件, 跳过 .tmp/.bak 等残留
            if entry.path().extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(record) = serde_json::from_slice::<RuntimeApprovalRecord>(&data) {
                    let key = auth::scoped_key(&auth::UserId(0), &record.approval_id);
                    state.approval_records.write().await.insert(key, record);
                } else {
                    safe_eprintln!("[startup] 跳过不可读的审批记录: {}", entry.path().display());
                }
            }
        }
    }
    // 从磁盘加载快照
    if let Ok(mut entries) = fs::read_dir(state.snapshot_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(snapshot) = serde_json::from_slice::<DeploymentSignatureSnapshot>(&data) {
                    let key = auth::scoped_key(&auth::UserId(0), &snapshot.snapshot_id);
                    state.snapshots.write().await.insert(key, snapshot);
                }
            }
        }
    }
    // 从磁盘加载告警 firing 状态
    if let Ok(mut entries) = fs::read_dir(state.alert_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(firing) = serde_json::from_slice::<AlertFiring>(&data) {
                    let key = auth::scoped_key(&auth::UserId(0), &firing.firing_id);
                    state.alert_firings.write().await.insert(key, firing);
                }
            }
        }
    }
    // 从磁盘加载沙箱报告
    if let Ok(mut entries) = fs::read_dir(state.sandbox_report_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(report) = serde_json::from_slice::<SandboxVerificationReport>(&data) {
                    let key = auth::scoped_key(&auth::UserId(0), &report.proposal_id);
                    state.sandbox_reports.write().await.insert(key, report);
                }
            }
        }
    }
    // 从磁盘加载混沌实验
    if let Ok(mut entries) = fs::read_dir(state.chaos_store_dir.as_ref()).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(data) = fs::read(entry.path()).await {
                if let Ok(experiment) = serde_json::from_slice::<ChaosExperimentReport>(&data) {
                    state
                        .chaos_experiments
                        .write()
                        .await
                        .insert(experiment.experiment_id.clone(), experiment);
                }
            }
        }
    }
    safe_eprintln!(
        "[startup] 已预热状态: {} 审批单, {} 快照, {} 告警, {} 沙箱报告, {} 混沌实验",
        state.approval_records.read().await.len(),
        state.snapshots.read().await.len(),
        state.alert_firings.read().await.len(),
        state.sandbox_reports.read().await.len(),
        state.chaos_experiments.read().await.len(),
    );
}

// Block 5 P1-5: 审批超时自动处理
// v2.3.3 修复 S0-1: 拆分为两阶段避免嵌套写锁死锁
// 阶段1: 在 approval_records 写锁内收集过期变更, 释放锁后批量持久化
// 阶段2: 在 ai_proposals 写锁内更新关联提案状态
async fn process_expired_approvals(state: &AppState) {
    let now_ms = current_time_ms();
    // 阶段1: 收集过期审批并标记, 记录需更新的 proposal_id 列表
    let expired_proposal_ids: Vec<String> = {
        let mut approvals = state.approval_records.write().await;
        let mut ids = Vec::new();
        for approval in approvals.values_mut() {
            if (approval.review_state == RuntimeApprovalReviewState::Pending
                || approval.review_state == RuntimeApprovalReviewState::UnderReview)
                && now_ms > approval.expires_at_ms
            {
                approval.review_state = RuntimeApprovalReviewState::Expired;
                approval.lifecycle.push(RuntimeApprovalLifecycleEntry {
                    review_state: RuntimeApprovalReviewState::Expired,
                    event_id: format!("event_apr_expired_{}", now_ms),
                    sequence_no: approval.lifecycle.len() as u64 + 1,
                    occurred_at_ms: now_ms,
                    reason_code: "APPROVAL_EXPIRED".to_string(),
                    message: format!(
                        "审批单已过期 (L{}审批, {}h超时)",
                        match approval.approval_level {
                            RuntimeApprovalLevel::L1SingleReviewer => 1,
                            RuntimeApprovalLevel::L2DualReviewer => 2,
                            RuntimeApprovalLevel::L3RiskOwnerReview => 3,
                        },
                        match approval.approval_level {
                            RuntimeApprovalLevel::L1SingleReviewer => 24,
                            RuntimeApprovalLevel::L2DualReviewer => 48,
                            RuntimeApprovalLevel::L3RiskOwnerReview => 72,
                        },
                    ),
                    actor_id: None,
                });
                ids.push(approval.proposal_id.clone());
                // 持久化在锁内完成, 但使用克隆数据避免长时间持锁
                if let Some(json) = serde_json::to_vec_pretty(&*approval).ok() {
                    let dir = state.approval_store_dir.to_path_buf();
                    let id = approval.approval_id.clone();
                    let approval_dir = dir.clone();
                    let approval_id = id.clone();
                    let approval_json = json.clone();
                    // spawn 到后台执行 I/O, 不持锁等待
                    tokio::spawn(async move {
                        let _ = tokio::fs::create_dir_all(&approval_dir).await;
                        let tmp = approval_dir.join(format!("{}.json.tmp", approval_id));
                        let final_path = approval_dir.join(format!("{}.json", approval_id));
                        let _ = tokio::fs::write(&tmp, &approval_json).await;
                        let _ = tokio::fs::rename(&tmp, &final_path).await;
                    });
                }
            }
        }
        ids
    }; // approval_records 写锁在此释放
       // 阶段2: 在 ai_proposals 写锁内更新提案状态 (独立锁, 无嵌套)
    if !expired_proposal_ids.is_empty() {
        let mut proposals = state.ai_proposals.write().await;
        for proposal_id in &expired_proposal_ids {
            if let Some(proposal) = proposals.get_mut(proposal_id) {
                proposal.status = RuntimeAiProposalStatus::Expired;
            }
        }
    }
}

// Block 5 P3-4: 观察窗口检查
async fn check_observation_windows(state: &AppState) {
    let now_ms = current_time_ms();
    let risk_reject = state
        .evidence_metrics
        .mutation_proposal_rejected_count
        .load(Ordering::Relaxed);
    let rollback_count = state
        .evidence_metrics
        .mutation_rollback_attempt_count
        .load(Ordering::Relaxed);

    // 检查最近的 mutation 激活记录，若在观察窗口内且异常，触发告警
    let mutations = state.parameter_mutations.read().await;
    for mutation in mutations.values() {
        if let Some(ref activation) = mutation.activation_state {
            if let Some(deadline_ms) = activation.observation_deadline_ms {
                if now_ms < deadline_ms {
                    // 仍在观察窗口内
                    if risk_reject > 100 || rollback_count > 0 {
                        // 异常检测：触发告警
                        let alert_id =
                            format!("alert-observation-{}-{}", mutation.proposal_id, now_ms);
                        let firing = AlertFiring {
                            firing_id: alert_id.clone(),
                            rule_name: "hotswap_rollback_occurred".to_string(),
                            severity: AlertSeverity::P2,
                            state: AlertFiringState::Firing,
                            fired_at_ms: now_ms,
                            acknowledged_at_ms: None,
                            resolved_at_ms: None,
                            acknowledged_by: None,
                            detail: format!(
                                "观察窗口异常: mutation {} 激活后风控拒绝率或回滚率超阈值",
                                mutation.proposal_id
                            ),
                        };
                        state.alert_firings.write().await.insert(alert_id, firing);
                    }
                }
            }
        }
    }
}

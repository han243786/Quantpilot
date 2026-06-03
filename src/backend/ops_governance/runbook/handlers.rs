use crate::*;

// ── Runbook: 已知故障场景诊断与恢复手册 ──
// Block 5: 6 类故障场景，含诊断步骤、恢复命令、验证标准

pub(super) fn register_runbook_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/runbook", get(list_scenarios))
        .route("/api/v1/runbook/:scenario_id", get(get_scenario))
}

fn build_default_runbook() -> Vec<RunbookScenario> {
    vec![
        // 场景 1: 数据源长时间不可用
        RunbookScenario {
            scenario_id: "data_source_unavailable".to_string(),
            name: "数据源长时间不可用".to_string(),
            symptoms: vec![
                "data_freshness_p95_ms（P95 数据新鲜度，毫秒） 持续上升".to_string(),
                "DataStale 事件产生".to_string(),
            ],
            severity: AlertSeverity::P1,
            diagnostic_steps: vec![
                RunbookDiagnosticStep {
                    step_number: 1,
                    description: "确认受影响数据源".to_string(),
                    api_call: Some("GET /api/health/data-sources".to_string()),
                    expected: Some("返回各数据源状态，定位不可用源".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 2,
                    description: "检查交易所状态页".to_string(),
                    api_call: None,
                    expected: Some("确认是否为交易所侧故障".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 3,
                    description: "检查网络连通性".to_string(),
                    api_call: None,
                    expected: Some("ping/traceroute 到交易所 API".to_string()),
                },
            ],
            recovery_steps: vec![
                RunbookRecoveryStep {
                    step_number: 1,
                    condition: "freshness > 3x poll_interval 且 < 5min".to_string(),
                    action: "观察，暂不干预".to_string(),
                },
                RunbookRecoveryStep {
                    step_number: 2,
                    condition: "freshness > 5min 持续".to_string(),
                    action: "手动暂停受影响 run；数据恢复后验证 freshness < 阈值 2min -> 手动恢复 run"
                        .to_string(),
                },
            ],
            verification: "data_freshness_p95_ms（P95 数据新鲜度，毫秒） 恢复正常，ExecutionPlanned 恢复产生".to_string(),
        },
        // 场景 2: 风控拒绝率异常飙升
        RunbookScenario {
            scenario_id: "risk_reject_rate_spike".to_string(),
            name: "风控拒绝率异常飙升".to_string(),
            symptoms: vec![
                "risk_reject_rate_5m（5 分钟风控拒绝率） > 90%".to_string(),
                "RiskRejected 事件激增".to_string(),
            ],
            severity: AlertSeverity::P2,
            diagnostic_steps: vec![
                RunbookDiagnosticStep {
                    step_number: 1,
                    description: "检查最近参数变更记录".to_string(),
                    api_call: Some("GET /api/runtime/mutations".to_string()),
                    expected: Some("确认是否有最近激活的参数变更".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 2,
                    description: "检查最近 AI 提案激活记录".to_string(),
                    api_call: Some("GET /api/runtime/ai-proposals?status=approved".to_string()),
                    expected: Some("确认是否有 AI 提案被激活".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 3,
                    description: "检查投资组合敞口是否超限".to_string(),
                    api_call: None,
                    expected: Some("对比当前持仓与风控限额".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 4,
                    description: "对比 baseline 与当前风控参数".to_string(),
                    api_call: None,
                    expected: Some("找出变更的差异点".to_string()),
                },
            ],
            recovery_steps: vec![
                RunbookRecoveryStep {
                    step_number: 1,
                    condition: "因参数变更导致".to_string(),
                    action: "POST /api/v1/runtime/mutations/:id/rollback 回滚到上一 generation"
                        .to_string(),
                },
                RunbookRecoveryStep {
                    step_number: 2,
                    condition: "因市场剧烈波动".to_string(),
                    action: "切换风险模式为 REDUCE_ONLY（仅减仓模式）".to_string(),
                },
            ],
            verification: "回滚后 30s 内风控拒绝率恢复正常水平".to_string(),
        },
        // 场景 3: 事件序列断裂
        RunbookScenario {
            scenario_id: "event_sequence_break".to_string(),
            name: "事件序列断裂".to_string(),
            symptoms: vec![
                "EventGapDetected（事件断裂检测） 事件产生".to_string(),
                "event_orphan_total 计数增长".to_string(),
            ],
            severity: AlertSeverity::P1,
            diagnostic_steps: vec![
                RunbookDiagnosticStep {
                    step_number: 1,
                    description: "检查 sequence_no 连续性".to_string(),
                    api_call: Some("GET /api/runs/:run_id/events".to_string()),
                    expected: Some("确认事件的 sequence_no 是否存在跳跃".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 2,
                    description: "检查事件日志写入是否异常".to_string(),
                    api_call: None,
                    expected: Some("查看磁盘 I/O 和日志文件".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 3,
                    description: "确认是否有未提交事务".to_string(),
                    api_call: None,
                    expected: Some("检查持久化层状态".to_string()),
                },
            ],
            recovery_steps: vec![
                RunbookRecoveryStep {
                    step_number: 1,
                    condition: "检测到事件缺口".to_string(),
                    action: "标记该 run 为回放不可信".to_string(),
                },
                RunbookRecoveryStep {
                    step_number: 2,
                    condition: "短暂中断".to_string(),
                    action: "等待自动恢复".to_string(),
                },
                RunbookRecoveryStep {
                    step_number: 3,
                    condition: "持久中断".to_string(),
                    action: "手动停止 run + 创建新 run".to_string(),
                },
            ],
            verification: "新 run 的 sequence_no 严格递增，无 EventGapDetected（事件断裂检测）".to_string(),
        },
        // 场景 4: 沙箱验证超时
        RunbookScenario {
            scenario_id: "sandbox_verification_timeout".to_string(),
            name: "沙箱验证超时".to_string(),
            symptoms: vec![
                "AI 提案的沙箱验证超过 5min 未完成".to_string(),
                "sandbox_verification_timeout 告警触发".to_string(),
            ],
            severity: AlertSeverity::P2,
            diagnostic_steps: vec![
                RunbookDiagnosticStep {
                    step_number: 1,
                    description: "检查沙箱进程是否存活".to_string(),
                    api_call: Some("GET /api/runtime/backtests/:id".to_string()),
                    expected: Some("确认回测任务状态".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 2,
                    description: "检查事件切片是否过大".to_string(),
                    api_call: None,
                    expected: Some("确认回放窗口的事件数量".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 3,
                    description: "检查磁盘 IO 是否饱和".to_string(),
                    api_call: None,
                    expected: Some("系统资源监控".to_string()),
                },
            ],
            recovery_steps: vec![
                RunbookRecoveryStep {
                    step_number: 1,
                    condition: "首次超时".to_string(),
                    action: "取消当前验证任务".to_string(),
                },
                RunbookRecoveryStep {
                    step_number: 2,
                    condition: "事件切片过大".to_string(),
                    action: "缩减回放窗口 (默认 30d -> 14d) 重试".to_string(),
                },
                RunbookRecoveryStep {
                    step_number: 3,
                    condition: "重试仍超时".to_string(),
                    action: "标记提案为验证不可用，转人工评估".to_string(),
                },
            ],
            verification: "沙箱验证在缩减窗口内完成，产出完整报告".to_string(),
        },
        // 场景 5: 磁盘水位告警
        RunbookScenario {
            scenario_id: "disk_watermark_alert".to_string(),
            name: "磁盘水位告警".to_string(),
            symptoms: vec![
                "storage_watermark_ratio > 85%".to_string(),
                "StorageWatermarkExceeded 事件".to_string(),
            ],
            severity: AlertSeverity::P1,
            diagnostic_steps: vec![
                RunbookDiagnosticStep {
                    step_number: 1,
                    description: "检查各存储层占用".to_string(),
                    api_call: None,
                    expected: Some("列出 storage/ 各子目录大小".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 2,
                    description: "确认压缩任务是否正常运行".to_string(),
                    api_call: None,
                    expected: Some("检查 report compaction 状态".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 3,
                    description: "确认 TTL（生存时间）淘汰是否触发".to_string(),
                    api_call: None,
                    expected: Some("检查过期数据清理".to_string()),
                },
            ],
            recovery_steps: vec![
                RunbookRecoveryStep {
                    step_number: 1,
                    condition: "85% 水位".to_string(),
                    action: "手动触发压缩任务 + 停止 debug（调试）写入".to_string(),
                },
                RunbookRecoveryStep {
                    step_number: 2,
                    condition: "90% 水位".to_string(),
                    action: "采样 DataUpdated + 暂停新 run".to_string(),
                },
                RunbookRecoveryStep {
                    step_number: 3,
                    condition: "95% 水位".to_string(),
                    action: "强制清空热层 ring buffer（环形缓冲区） (保留 key/summary)".to_string(),
                },
            ],
            verification: "水位降至 70% 以下，压缩任务正常完成".to_string(),
        },
        // 场景 6: 热插拔回滚
        RunbookScenario {
            scenario_id: "hotswap_rollback".to_string(),
            name: "热插拔回滚".to_string(),
            symptoms: vec![
                "HotSwapRollback（热插拔回滚事件） 事件产生".to_string(),
                "deployment_revision（部署修订号） 未变更".to_string(),
            ],
            severity: AlertSeverity::P1,
            diagnostic_steps: vec![
                RunbookDiagnosticStep {
                    step_number: 1,
                    description: "检查回滚原因码".to_string(),
                    api_call: None,
                    expected: Some(
                        "确认 compatibility（兼容性检查）/ safe_window（安全窗口）/ shadow_replay（影子回放）/ observation（观察窗口）中哪个触发".to_string(),
                    ),
                },
                RunbookDiagnosticStep {
                    step_number: 2,
                    description: "检查回滚前快照是否完整".to_string(),
                    api_call: Some("GET /api/v1/snapshots".to_string()),
                    expected: Some("确认是否有可用快照".to_string()),
                },
                RunbookDiagnosticStep {
                    step_number: 3,
                    description: "检查事件日志中回滚步骤详情".to_string(),
                    api_call: None,
                    expected: Some("追踪回滚事件序列".to_string()),
                },
            ],
            recovery_steps: vec![
                RunbookRecoveryStep {
                    step_number: 1,
                    condition: "回滚已自动完成".to_string(),
                    action: "确认已恢复到 pre-swap（热插拔前）deployment_revision（部署修订号）".to_string(),
                },
                RunbookRecoveryStep {
                    step_number: 2,
                    condition: "需要重新尝试".to_string(),
                    action: "分析回滚原因并修复后重试".to_string(),
                },
            ],
            verification: "原 deployment_revision（部署修订号） 继续正常运行，无事件断裂".to_string(),
        },
    ]
}

async fn list_scenarios() -> Result<Json<Vec<RunbookScenario>>, (StatusCode, String)> {
    Ok(Json(build_default_runbook()))
}

async fn get_scenario(
    Path(scenario_id): Path<String>,
) -> Result<Json<RunbookScenario>, (StatusCode, String)> {
    let scenarios = build_default_runbook();
    if let Some(scenario) = scenarios.into_iter().find(|s| s.scenario_id == scenario_id) {
        return Ok(Json(scenario));
    }
    Err(json_bad_request(
        "not_found",
        format!("故障场景 '{}' 不存在", scenario_id),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_runbook_has_six_scenarios() {
        let scenarios = super::build_default_runbook();
        assert_eq!(scenarios.len(), 6);
    }

    #[test]
    fn each_scenario_has_diagnostic_and_recovery_steps() {
        for scenario in &super::build_default_runbook() {
            assert!(
                !scenario.diagnostic_steps.is_empty(),
                "scenario {} has no diagnostic steps",
                scenario.scenario_id
            );
            assert!(
                !scenario.recovery_steps.is_empty(),
                "scenario {} has no recovery steps",
                scenario.scenario_id
            );
            assert!(
                !scenario.verification.is_empty(),
                "scenario {} has no verification",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn all_scenario_ids_are_unique() {
        let scenarios = super::build_default_runbook();
        let ids: Vec<_> = scenarios.iter().map(|s| &s.scenario_id).collect();
        let unique: std::collections::BTreeSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }
}

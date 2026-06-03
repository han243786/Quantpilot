use crate::{AlertRule, AlertSeverity};

pub(super) fn default_alert_rules() -> Vec<AlertRule> {
    vec![
        AlertRule {
            rule_name: "data_freshness_critical".to_string(),
            description: "P95 freshness > 3x poll_interval 持续 5min".to_string(),
            trigger_condition: "data_freshness_p95_ms > 3 * poll_interval_ms AND duration >= 300s".to_string(),
            severity: AlertSeverity::P1,
            action: "数据新鲜度 P95 超过 3 倍轮询间隔且持续 5 分钟以上。".to_string(),
            enabled: true,
            resolve_condition: Some("data_freshness_p95_ms < poll_interval_ms".to_string()),
        },
        AlertRule {
            rule_name: "event_orphan_detected".to_string(),
            description: "任意 event_orphan_total 增长".to_string(),
            trigger_condition: "event_orphan_total > 0".to_string(),
            severity: AlertSeverity::P1,
            action: "检测到事件序列断裂。将当前运行标记为审计不可信。".to_string(),
            enabled: true,
            resolve_condition: Some("event_orphan_total == 0".to_string()),
        },
        AlertRule {
            rule_name: "risk_reject_rate_spike".to_string(),
            description: "5min 拒绝率 > 90% 且样本数 > 50".to_string(),
            trigger_condition: "risk_reject_rate_5m > 0.90 AND sample_count > 50".to_string(),
            severity: AlertSeverity::P2,
            action: "风控拒绝率 5 分钟内超过 90%（样本数 > 50）。通知策略负责人，检查最近参数变更记录（GET /api/runtime/mutations），对比当前风控限额与持仓敞口。如因参数变更导致，回滚最近一次变更。".to_string(),
            enabled: true,
            resolve_condition: Some("risk_reject_rate_5m < 0.50 AND sample_count < 10".to_string()),
        },
        AlertRule {
            rule_name: "replay_divergence_detected".to_string(),
            description: "replay_divergence_total 增长".to_string(),
            trigger_condition: "replay_divergence_total > 0".to_string(),
            severity: AlertSeverity::P1,
            action: "回放差异增长（replay_divergence_total > 0）。归档当前回放差异证据（事件日志 + 权益曲线对比），通知值班人员 + QA 分析根因。".to_string(),
            enabled: true,
            resolve_condition: Some("replay_divergence_total == 0".to_string()),
        },
        AlertRule {
            rule_name: "ai_proposal_reject_rate_high".to_string(),
            description: "24h 拒绝率 > 80% 且提案数 > 5".to_string(),
            trigger_condition: "ai_proposal_reject_rate_24h > 0.80 AND proposal_count > 5"
                .to_string(),
            severity: AlertSeverity::P2,
            action: "AI 提案 24 小时拒绝率超过 80%（提案数 > 5）。检查最近提案的 static_check 报告，如模型输出持续低质量，暂停 AI 提案 24 小时。".to_string(),
            enabled: true,
            resolve_condition: Some("ai_proposal_reject_rate_24h < 0.30".to_string()),
        },
        AlertRule {
            rule_name: "sandbox_verification_timeout".to_string(),
            description: "沙箱验证超 5min 未完成".to_string(),
            trigger_condition: "sandbox_verification_duration_ms > 300000".to_string(),
            severity: AlertSeverity::P2,
            action: "沙箱验证超过 5 分钟未完成。取消本次验证，通知提案提交者优化策略参数后重新提交。".to_string(),
            enabled: true,
            resolve_condition: Some("sandbox_verification_duration_ms < 30000".to_string()),
        },
        AlertRule {
            rule_name: "storage_watermark_critical".to_string(),
            description: "存储总大小超过 450MB (90% 阈值)".to_string(),
            trigger_condition: "disk_watermark_ratio > 0.90".to_string(),
            severity: AlertSeverity::P1,
            action: "存储总大小超过 450MB（90% 配额阈值）。立即执行启动清理流程：删除所有过期瞬间/暂时数据，暂停新的非长期写入。".to_string(),
            enabled: true,
            resolve_condition: Some("disk_watermark_ratio < 0.80".to_string()),
        },
        AlertRule {
            rule_name: "approval_expiry_warning".to_string(),
            description: "审批单 4h 内到期未处理".to_string(),
            trigger_condition: "approval_expires_in_ms < 14400000".to_string(),
            severity: AlertSeverity::P3,
            action: "审批单将在 4 小时内到期且未被处理。提醒审批人尽快审阅待处理审批（GET /api/v1/approvals?status=pending）。".to_string(),
            enabled: true,
            resolve_condition: Some("approval_expires_in_ms > 14400000".to_string()),
        },
        AlertRule {
            rule_name: "hotswap_rollback_occurred".to_string(),
            description: "热插拔回滚发生".to_string(),
            trigger_condition: "hotswap_rollback_count > 0".to_string(),
            severity: AlertSeverity::P1,
            action: "热插拔回滚发生。通知值班人员 + 策略负责人，冻结 AI 提案 24 小时，检查兼容性报告和 safe_window 状态确认回滚原因。".to_string(),
            enabled: true,
            resolve_condition: Some("hotswap_rollback_count == 0 AND 24h 窗口已过".to_string()),
        },
        AlertRule {
            rule_name: "capability_hash_mismatch".to_string(),
            description: "compile/runtime hash 不一致".to_string(),
            trigger_condition: "capability_hash_compile != capability_hash_runtime".to_string(),
            severity: AlertSeverity::P1,
            action: "编译时与运行时的 capability 哈希不一致。系统能力合约可能已被篡改或版本不匹配。立即阻断启动，通知值班人员检查部署版本和 capability 签名。".to_string(),
            enabled: true,
            resolve_condition: Some("capability_hash_compile == capability_hash_runtime".to_string()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_alert_rules_has_ten_rules() {
        let rules = super::default_alert_rules();
        assert_eq!(rules.len(), 10);
        assert!(rules.iter().all(|r| !r.rule_name.is_empty()));
    }

    #[test]
    fn p1_rules_include_data_freshness_and_storage() {
        let rules = super::default_alert_rules();
        let p1_rules: Vec<_> = rules
            .iter()
            .filter(|r| matches!(r.severity, AlertSeverity::P1))
            .collect();
        let names: Vec<_> = p1_rules.iter().map(|r| &r.rule_name).collect();
        assert!(names.contains(&&"data_freshness_critical".to_string()));
        assert!(names.contains(&&"storage_watermark_critical".to_string()));
    }

    #[test]
    fn all_rules_have_severity_and_action() {
        let rules = super::default_alert_rules();
        for rule in &rules {
            assert!(
                !rule.action.is_empty(),
                "rule {} has no action",
                rule.rule_name
            );
        }
    }
}

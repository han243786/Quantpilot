const SUPPORTED_INTENT_MODULES: &str =
    "double_ma/ma_deviation/rsi/macd/momentum/zscore/spread_observer";

pub(super) fn bail_unsupported_intent(module_key: &str) -> anyhow::Result<()> {
    anyhow::bail!(
        "不支持的意图模块 '{}': 当前版本仅支持 {}。请升级到支持该模块的版本。",
        module_key,
        SUPPORTED_INTENT_MODULES
    );
}

#[cfg(test)]
mod tests {
    use super::bail_unsupported_intent;

    #[test]
    fn unsupported_intent_failure_message_stays_stable() {
        let error = bail_unsupported_intent("builtin.intent.unknown").unwrap_err();

        assert_eq!(
            error.to_string(),
            "不支持的意图模块 'builtin.intent.unknown': 当前版本仅支持 double_ma/ma_deviation/rsi/macd/momentum/zscore/spread_observer。请升级到支持该模块的版本。"
        );
    }
}

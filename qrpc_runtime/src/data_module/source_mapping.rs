use qrpc_core::{CoreStrategyIr, DataKind, DataSourceConfig, Exchange, Symbol};
use qrpc_core_ir::DataBindingKind;

pub(crate) fn data_sources_from_core_ir(core_ir: &CoreStrategyIr) -> Vec<DataSourceConfig> {
    core_ir
        .data_bindings
        .iter()
        .filter_map(|binding| {
            let exchange = binding
                .source_hints
                .get("exchange")
                .and_then(|value| parse_exchange_hint(value));
            let symbol = binding
                .source_hints
                .get("symbol")
                .and_then(|value| parse_symbol_hint(value));
            match (exchange, symbol) {
                (Some(exchange), Some(symbol)) => Some(DataSourceConfig {
                    data_id: binding.data_id.clone(),
                    exchange,
                    symbol,
                    market_type: qrpc_core::MarketType::Spot,
                    kind: match binding.kind {
                        DataBindingKind::KlineSeries => DataKind::KlineSeries,
                        DataBindingKind::Quote => DataKind::Quote,
                    },
                    days: matches!(binding.kind, DataBindingKind::KlineSeries).then_some(200),
                    interval: matches!(binding.kind, DataBindingKind::KlineSeries).then(|| {
                        binding
                            .source_hints
                            .get("timeframe")
                            .cloned()
                            .unwrap_or_else(|| "1d".to_string())
                    }),
                    ping_enabled: binding
                        .source_hints
                        .get("ping_enabled")
                        .and_then(|value| parse_bool_hint(value))
                        .unwrap_or(false),
                    request_interval_ms: binding
                        .source_hints
                        .get("request_interval_ms")
                        .and_then(|value| parse_u64_hint(value)),
                    enabled: true,
                }),
                _ => None,
            }
        })
        .collect()
}

fn parse_exchange_hint(value: &str) -> Option<Exchange> {
    match value.to_ascii_lowercase().as_str() {
        "binance" => Some(Exchange::Binance),
        "okx" => Some(Exchange::Okx),
        _ => None,
    }
}

fn parse_symbol_hint(value: &str) -> Option<Symbol> {
    let value = value.trim();
    (!value.is_empty()).then(|| Symbol::parse(value))
}

fn parse_bool_hint(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_u64_hint(value: &str) -> Option<u64> {
    value.trim().parse::<u64>().ok()
}

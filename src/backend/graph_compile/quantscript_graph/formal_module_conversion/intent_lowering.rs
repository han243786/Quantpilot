mod double_ma_lowering;
mod ma_deviation_lowering;
mod macd_lowering;
mod momentum_lowering;
mod rsi_lowering;
mod shared_intent_context;
mod spread_observer_lowering;
mod zscore_lowering;

use serde_json::Value;

pub(super) fn append_intent_lowering_lines(
    nodes: &[Value],
    edges: &[Value],
    qs_lines: &mut Vec<String>,
) -> anyhow::Result<()> {
    // Add indicator/emit calls for intent nodes
    for node in nodes {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or("");
        if node_type == "intent" {
            let ctx = shared_intent_context::resolve_intent_lowering_context(node, edges);

            match ctx.module_key {
                "builtin.intent.double_ma" => {
                    double_ma_lowering::append_double_ma_lowering_lines(
                        ctx.cfg,
                        &ctx.source_var,
                        ctx.instrument,
                        qs_lines,
                    );
                }
                "builtin.intent.rsi" => {
                    rsi_lowering::append_rsi_lowering_lines(
                        ctx.node_id,
                        ctx.cfg,
                        &ctx.source_var,
                        ctx.instrument,
                        qs_lines,
                    );
                }
                "builtin.intent.ma_deviation" => {
                    ma_deviation_lowering::append_ma_deviation_lowering_lines(
                        ctx.cfg,
                        &ctx.source_var,
                        ctx.instrument,
                        qs_lines,
                    );
                }
                "builtin.intent.macd" => {
                    macd_lowering::append_macd_lowering_lines(
                        ctx.cfg,
                        &ctx.source_var,
                        ctx.instrument,
                        qs_lines,
                    );
                }
                "builtin.intent.momentum" => {
                    momentum_lowering::append_momentum_lowering_lines(
                        ctx.node_id,
                        ctx.cfg,
                        &ctx.source_var,
                        ctx.instrument,
                        qs_lines,
                    );
                }
                "builtin.intent.zscore" => {
                    zscore_lowering::append_zscore_lowering_lines(
                        ctx.node_id,
                        ctx.cfg,
                        &ctx.source_var,
                        ctx.instrument,
                        qs_lines,
                    );
                }
                "builtin.intent.spread_observer" => {
                    spread_observer_lowering::append_spread_observer_lowering_lines(
                        node,
                        edges,
                        ctx.cfg,
                        ctx.node_id,
                        &ctx.source_var,
                        ctx.instrument,
                        qs_lines,
                    );
                }
                _ => {
                    // v2.3.3 修复 S0-5: 未知意图模块键不再静默丢弃, 返回明确错误
                    let supported =
                        "double_ma/ma_deviation/rsi/macd/momentum/zscore/spread_observer";
                    anyhow::bail!(
                        "不支持的意图模块 '{}': 当前版本仅支持 {}。请升级到支持该模块的版本。",
                        ctx.module_key,
                        supported
                    );
                }
            }
        }
    }

    Ok(())
}

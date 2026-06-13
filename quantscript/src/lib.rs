mod analysis_diagnostics;
mod evaluator;
mod legacy_config_compat;
mod lowering;
mod resolve;
mod syntax_ast_surface;
mod test_plan;
mod v4_static_audit;

use anyhow::Result;
use qrpc_core::RuntimeProtocolCoreConfig;

pub use analysis_diagnostics::{
    analyze_script_module, Diagnostic, DiagnosticSeverity, ScriptAnalysis, Span, SpanContext,
};
pub use evaluator::normalize_script_module;
#[allow(deprecated)]
pub use legacy_config_compat::{
    compile_program_to_config, compile_quant_script, compile_quant_script_file, parse_quant_script,
    parse_quant_script_config, parse_quant_script_source, AgentSection, DataSection, IntentSection,
    QuantScriptProgram, QuantScriptSource, RiskSection, RuntimeSection,
};
pub use lowering::{
    lower_script_to_runtime_config, lower_script_to_runtime_config_with_context,
    InstrumentPoolEligibilityRule, InstrumentPoolFeatureDef, InstrumentPoolRebalanceRule,
    InstrumentPoolSelectionKey, InstrumentPoolSelectionRule, InstrumentPoolSourceSpec,
    InstrumentPoolSpec, InstrumentPoolValue, InstrumentPoolWeightingRule, LoweringContext,
};
pub use resolve::{
    classify_builtin_math_name, classify_member_mutation_name, classify_series_capability_name,
    expr_semantic_key, lower_script_to_typed_hir, ChangeHelperKind, KnownIndicatorHelperKind,
    MovingAverageHelperKind, ResolveResult, ResolvedBuiltinMathKind, ResolvedCallable,
    ResolvedCallableKind, ResolvedChangeSmoothingKind, ResolvedExprSemantic,
    ResolvedFetchSourceKind, ResolvedFunction, ResolvedManualIndicatorFormula,
    ResolvedMemberMutationKind, ResolvedSeriesBoundaryKind, ResolvedSeriesCapabilityKind,
    ResolvedSeriesViewKind, ResolvedWindowAggregateKind, ResolvedWindowAggregateView,
    RsiHelperKind,
};
pub use syntax_ast_surface::{
    parse_expr, parse_quant_script_module, BinaryOp, CallArg, Expr, FunctionDecl, ImportDecl,
    ImportName, Item, MatchArm, MatchArmBody, Param, ScriptModule, StepBlock, Stmt, TestAction,
    TestBlock, TestParamValue, UnaryOp,
};
pub use syntax_ast_surface::{
    parse_type_annotation, DefId, ExprId, HirBindingPattern, HirCallArg, HirExpr, HirExprKind,
    HirFunction, HirImport, HirImportName, HirLetStmt, HirMatchArm, HirMatchArmBody, HirParam,
    HirStmt, Type, TypeArena, TypeId, TypedHirModule,
};
pub use test_plan::{
    extract_test_plan, split_test_items, TestActionDef, TestParamValueDef, TestPlan, TestStep,
};
pub use v4_static_audit::{
    audit_v4_quant_script_static, build_v4_qs_runtime_handoff, V4QsRuntimeHandoffReport,
    V4QsStaticAuditReport, V4QsStaticAuditVerdict, V4_QS_RUNTIME_HANDOFF_REPORT_VERSION,
};

pub(crate) use analysis_diagnostics::{analysis, diagnostics};
pub(crate) use syntax_ast_surface::{hir, script, types};

pub fn parse_formal_quant_script_config(input: &str) -> Result<RuntimeProtocolCoreConfig> {
    let module = parse_quant_script_module(input)?;
    lower_script_to_runtime_config(&module)
}

pub fn parse_formal_quant_script_typed_hir(input: &str) -> Result<ResolveResult> {
    let module = parse_quant_script_module(input)?;
    Ok(lower_script_to_typed_hir(&module))
}

pub fn analyze_formal_quant_script(input: &str) -> Result<ScriptAnalysis> {
    let module = parse_quant_script_module(input)?;
    let resolved = lower_script_to_typed_hir(&module);
    Ok(analyze_script_module(&module, &resolved))
}

pub fn extract_formal_instrument_pool_spec(
    module: &ScriptModule,
    context: &LoweringContext,
) -> Result<Option<InstrumentPoolSpec>> {
    let normalized = normalize_script_module(module)?;
    lowering::extract_instrument_pool_spec(&normalized, context)
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn lowers_formal_script_into_runtime_config() {
        let config = parse_formal_quant_script_config(
            r#"
import math
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=150)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();
        assert_eq!(config.data_sources.len(), 1);
        assert_eq!(config.intents.len(), 2);
    }

    #[test]
    fn reports_centered_window_lookahead_risk() {
        let analysis = analyze_formal_quant_script(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=200)?
    let smooth = rolling_mean(closes, window=20, center=true)
    if smooth > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        assert!(analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "QS0402" && diagnostic.message.contains("前视风险")
        }));
    }

    #[test]
    fn reports_insufficient_fetch_lookback_for_warmup() {
        let analysis = analyze_formal_quant_script(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=50)?
    let slow = closes[200..].sum() / 200
    if closes.last() > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        assert_eq!(analysis.required_warmup_bars, 200);
        assert!(analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "QS0501" && diagnostic.message.contains("预热不足: 策略至少需要 200")
        }));
    }

    #[test]
    fn reports_negative_series_index_lookahead_risk() {
        let analysis = analyze_formal_quant_script(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=50)?
    let latest = closes[-1]
    if latest > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        assert!(analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "QS0401" && diagnostic.message.contains("前视风险: 负数序列索引")
        }));
    }

    #[test]
    fn derives_warmup_from_direct_history_access() {
        let analysis = analyze_formal_quant_script(
            r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=10)?
    let prior = closes[14]
    if prior > 0 {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
}
"#,
        )
        .unwrap();

        assert_eq!(analysis.required_warmup_bars, 14);
        assert!(analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "QS0501" && diagnostic.message.contains("预热不足: 策略至少需要 14")
        }));
    }

    #[test]
    fn reports_non_trunk_control_flow_and_recursion_constructs() {
        let analysis = analyze_formal_quant_script(
            r#"
import data as market_data

  fn helper(series) {
      return helper(series)
  }
  
async fn strategy() {
    let closes = await fetch("BTCUSDT", interval="1d", lookback=50)?
    let unsafe_try = sma(closes, 20)?
    let mut out = []
    out.push(1)
    if fetch("BTCUSDT", interval="1d", lookback=20).ok() {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    for value in closes[20..] {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    }
    let i = 0
    while i < 1 {
        match closes[0] {
            _ => emit Intent("BUY", instrument="BTCUSDT", quantity=1.0),
        }
    }
}
"#,
        )
        .unwrap();

        assert!(analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QS0601"));
        assert!(analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QS0602"));
        assert!(analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QS0603"));
        assert!(analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QS0604"));
        assert!(analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QS0605"));
        assert!(analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QS0606"));
        assert!(analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QS0607"));
        assert!(analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QS0608"));
        assert!(analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QS0609"));
        assert!(analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QS0610"));
    }
}

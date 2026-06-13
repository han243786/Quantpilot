mod analysis;
mod diagnostics;
mod evaluator;
mod lowering;
mod resolve;
mod syntax_ast_surface;
mod test_plan;
mod v4_static_audit;

use anyhow::{anyhow, bail, Context, Result};
use qrpc_compiler::compile_runtime_protocol_config;
use qrpc_core::{
    AgentConfig, CompiledRuntimeProtocol, DataKind, DataSourceConfig, Exchange, IntentConfig,
    IntentKind, MarketType, RiskConfig, RuntimeProtocolCoreConfig, Symbol,
};
use std::collections::BTreeMap;
use std::path::Path;

pub use analysis::{analyze_script_module, ScriptAnalysis};
pub use diagnostics::{Diagnostic, DiagnosticSeverity, Span, SpanContext};
pub use evaluator::normalize_script_module;
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

pub(crate) use syntax_ast_surface::{hir, script, types};

#[derive(Debug, Clone, PartialEq)]
pub struct QuantScriptProgram {
    pub runtime: RuntimeSection,
    pub data_sources: Vec<DataSection>,
    pub intents: Vec<IntentSection>,
    pub agents: Vec<AgentSection>,
    pub risks: Vec<RiskSection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeSection {
    pub initial_cash_balance: f64,
    pub taker_fee_bps: f64,
    pub default_slippage_bps: f64,
    pub total_cost_buffer_bps: f64,
}

impl Default for RuntimeSection {
    fn default() -> Self {
        Self {
            initial_cash_balance: 100_000.0,
            taker_fee_bps: 10.0,
            default_slippage_bps: 5.0,
            total_cost_buffer_bps: 20.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataSection {
    pub id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub market_type: MarketType,
    pub kind: DataKind,
    pub days: Option<u32>,
    pub interval: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntentSection {
    pub id: String,
    pub name: String,
    pub kind: IntentKind,
    pub inputs: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentSection {
    pub id: String,
    pub name: String,
    pub intents: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RiskSection {
    pub id: String,
    pub name: String,
    pub agents: Vec<String>,
    pub max_total_leverage: f64,
    pub max_exchange_leverage: f64,
    pub min_action_interval_ms: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    String(String),
    Number(f64),
    Bool(bool),
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    Comma,
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    List(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuantScriptSource {
    #[deprecated(
        note = "legacy config-style QuantScript is kept only for compatibility; prefer formal QuantScript source via `parse_quant_script_module` or `parse_formal_quant_script_config`"
    )]
    Config(QuantScriptProgram),
    Script(ScriptModule),
}

#[deprecated(
    note = "legacy config-style QuantScript is kept only for compatibility; formal QuantScript is the supported product path"
)]
pub fn parse_quant_script(input: &str) -> Result<QuantScriptProgram> {
    let tokens = tokenize(input)?;
    Parser::new(tokens).parse_program()
}

#[deprecated(
    note = "legacy source-kind autodetection is kept only for compatibility; formal QuantScript is the supported product path"
)]
#[allow(deprecated)]
pub fn parse_quant_script_source(input: &str) -> Result<QuantScriptSource> {
    if looks_like_config(input) {
        Ok(QuantScriptSource::Config(parse_quant_script(input)?))
    } else if looks_like_script(input) {
        Ok(QuantScriptSource::Script(parse_quant_script_module(input)?))
    } else {
        bail!("无法确定 QuantScript 源码类型")
    }
}

#[deprecated(
    note = "legacy config-style QuantScript is kept only for compatibility; formal QuantScript is the supported product path"
)]
#[allow(deprecated)]
pub fn parse_quant_script_config(input: &str) -> Result<RuntimeProtocolCoreConfig> {
    let program = parse_quant_script(input)?;
    compile_program_to_config(&program)
}

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

#[deprecated(
    note = "legacy config-style QuantScript compilation is kept only for compatibility; formal QuantScript is the supported product path"
)]
#[allow(deprecated)]
pub fn compile_quant_script(input: &str) -> Result<CompiledRuntimeProtocol> {
    let config = parse_quant_script_config(input)?;
    compile_runtime_protocol_config(&config)
}

#[deprecated(
    note = "legacy config-style QuantScript compilation is kept only for compatibility; formal QuantScript is the supported product path"
)]
#[allow(deprecated)]
pub fn compile_quant_script_file(path: impl AsRef<Path>) -> Result<CompiledRuntimeProtocol> {
    let source = std::fs::read_to_string(path.as_ref())
        .with_context(|| format!("读取 QuantScript 文件失败: {}", path.as_ref().display()))?;
    compile_quant_script(&source)
}

pub fn compile_program_to_config(
    program: &QuantScriptProgram,
) -> Result<RuntimeProtocolCoreConfig> {
    Ok(RuntimeProtocolCoreConfig {
        data_sources: program
            .data_sources
            .iter()
            .map(|data| DataSourceConfig {
                data_id: data.id.clone(),
                exchange: data.exchange.clone(),
                symbol: data.symbol.clone(),
                market_type: data.market_type.clone(),
                kind: data.kind.clone(),
                days: data.days,
                interval: data.interval.clone(),
                ping_enabled: false,
                request_interval_ms: None,
                enabled: data.enabled,
            })
            .collect(),
        intents: program
            .intents
            .iter()
            .map(|intent| IntentConfig {
                intent_id: intent.id.clone(),
                name: intent.name.clone(),
                kind: intent.kind.clone(),
                input_data_ids: intent.inputs.clone(),
                params: BTreeMap::new(),
                enabled: intent.enabled,
            })
            .collect(),
        agents: program
            .agents
            .iter()
            .map(|agent| AgentConfig {
                agent_id: agent.id.clone(),
                name: agent.name.clone(),
                input_intent_ids: agent.intents.clone(),
                rebalance_symbols: vec![],
                rebalance_schedule: None,
                rebalance_allocation_kind: None,
                rebalance_rank_method: None,
                rebalance_score_normalize: None,
                rebalance_target_weights: vec![],
                params: BTreeMap::new(),
                enabled: agent.enabled,
            })
            .collect(),
        risks: program
            .risks
            .iter()
            .map(|risk| RiskConfig {
                risk_id: risk.id.clone(),
                name: risk.name.clone(),
                observed_agent_ids: risk.agents.clone(),
                max_position_ratio: 0.2,
                max_single_weight: None,
                max_concentration_ratio: None,
                max_symbol_net_exposure_ratio: None,
                max_portfolio_net_exposure_ratio: None,
                max_turnover: None,
                min_trade_weight: None,
                max_new_positions_per_rebalance: None,
                max_total_leverage: risk.max_total_leverage,
                max_exchange_leverage: risk.max_exchange_leverage,
                min_action_interval_ms: risk.min_action_interval_ms,
                enabled: risk.enabled,
            })
            .collect(),
        initial_cash_balance: program.runtime.initial_cash_balance,
        taker_fee_bps: program.runtime.taker_fee_bps,
        default_slippage_bps: program.runtime.default_slippage_bps,
        total_cost_buffer_bps: program.runtime.total_cost_buffer_bps,
    })
}

fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            '{' => {
                chars.next();
                tokens.push(Token::LBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RBrace);
            }
            '[' => {
                chars.next();
                tokens.push(Token::LBracket);
            }
            ']' => {
                chars.next();
                tokens.push(Token::RBracket);
            }
            ':' => {
                chars.next();
                tokens.push(Token::Colon);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            '"' => {
                chars.next();
                let mut content = String::new();
                while let Some(next) = chars.next() {
                    match next {
                        '"' => break,
                        '\\' => {
                            let escaped =
                                chars.next().ok_or_else(|| anyhow!("未终止的转义序列"))?;
                            content.push(match escaped {
                                'n' => '\n',
                                't' => '\t',
                                '"' => '"',
                                '\\' => '\\',
                                other => other,
                            });
                        }
                        other => content.push(other),
                    }
                }
                tokens.push(Token::String(content));
            }
            '#' => {
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next == '\n' {
                        break;
                    }
                }
            }
            '/' => {
                chars.next();
                if matches!(chars.peek(), Some('/')) {
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == '\n' {
                            break;
                        }
                    }
                } else {
                    bail!("QuantScript 中意外的 '/'");
                }
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            c if c.is_ascii_digit() || c == '-' => {
                let mut number = String::new();
                number.push(chars.next().expect("digit or '-' already peeked"));
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() || next == '.' {
                        number.push(chars.next().expect("number continuation already peeked"));
                    } else {
                        break;
                    }
                }
                let parsed = number
                    .parse::<f64>()
                    .with_context(|| format!("无效的数字字面量: {number}"))?;
                tokens.push(Token::Number(parsed));
            }
            _ => {
                let mut ident = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_alphanumeric() || matches!(next, '_' | '-' | '.') {
                        ident.push(
                            chars
                                .next()
                                .expect("identifier continuation already peeked"),
                        );
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "" => bail!("QuantScript 中意外的字符: {ch}"),
                    "true" => tokens.push(Token::Bool(true)),
                    "false" => tokens.push(Token::Bool(false)),
                    _ => tokens.push(Token::Ident(ident)),
                }
            }
        }
    }

    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    fn parse_program(&mut self) -> Result<QuantScriptProgram> {
        let mut runtime = RuntimeSection::default();
        let mut data_sources = Vec::new();
        let mut intents = Vec::new();
        let mut agents = Vec::new();
        let mut risks = Vec::new();

        while !self.is_eof() {
            match self.expect_ident("section keyword")?.as_str() {
                "runtime" => runtime = self.parse_runtime_section()?,
                "data" => data_sources.push(self.parse_data_section()?),
                "intent" => intents.push(self.parse_intent_section()?),
                "agent" => agents.push(self.parse_agent_section()?),
                "risk" => risks.push(self.parse_risk_section()?),
                other => bail!("不支持的 QuantScript 区块: {other}"),
            }
        }

        Ok(QuantScriptProgram {
            runtime,
            data_sources,
            intents,
            agents,
            risks,
        })
    }

    fn parse_runtime_section(&mut self) -> Result<RuntimeSection> {
        let fields = self.parse_block_fields()?;
        Ok(RuntimeSection {
            initial_cash_balance: field_number(&fields, "initial_cash_balance")?
                .unwrap_or(100_000.0),
            taker_fee_bps: field_number(&fields, "taker_fee_bps")?.unwrap_or(10.0),
            default_slippage_bps: field_number(&fields, "default_slippage_bps")?.unwrap_or(5.0),
            total_cost_buffer_bps: field_number(&fields, "total_cost_buffer_bps")?.unwrap_or(20.0),
        })
    }

    fn parse_data_section(&mut self) -> Result<DataSection> {
        let id = self.expect_ident("data id")?;
        let fields = self.parse_block_fields()?;
        Ok(DataSection {
            id,
            exchange: parse_exchange(&field_ident_required(&fields, "exchange")?)?,
            symbol: parse_symbol(&field_ident_required(&fields, "symbol")?)?,
            market_type: parse_market_type(&field_ident_required(&fields, "market_type")?)?,
            kind: parse_data_kind(&field_ident_required(&fields, "kind")?)?,
            days: field_number(&fields, "days")?.map(|value| value as u32),
            interval: field_string(&fields, "interval")?,
            enabled: field_bool(&fields, "enabled")?.unwrap_or(true),
        })
    }

    fn parse_intent_section(&mut self) -> Result<IntentSection> {
        let id = self.expect_ident("intent id")?;
        let fields = self.parse_block_fields()?;
        Ok(IntentSection {
            id,
            name: field_string_required(&fields, "name")?,
            kind: parse_intent_kind(&field_ident_required(&fields, "kind")?)?,
            inputs: field_list_required(&fields, "inputs")?,
            enabled: field_bool(&fields, "enabled")?.unwrap_or(true),
        })
    }

    fn parse_agent_section(&mut self) -> Result<AgentSection> {
        let id = self.expect_ident("agent id")?;
        let fields = self.parse_block_fields()?;
        Ok(AgentSection {
            id,
            name: field_string_required(&fields, "name")?,
            intents: field_list_required(&fields, "intents")?,
            enabled: field_bool(&fields, "enabled")?.unwrap_or(true),
        })
    }

    fn parse_risk_section(&mut self) -> Result<RiskSection> {
        let id = self.expect_ident("risk id")?;
        let fields = self.parse_block_fields()?;
        Ok(RiskSection {
            id,
            name: field_string_required(&fields, "name")?,
            agents: field_list_required(&fields, "agents")?,
            max_total_leverage: field_number(&fields, "max_total_leverage")?.unwrap_or(3.0),
            max_exchange_leverage: field_number(&fields, "max_exchange_leverage")?.unwrap_or(3.0),
            min_action_interval_ms: field_number(&fields, "min_action_interval_ms")?
                .unwrap_or(1_000.0) as u64,
            enabled: field_bool(&fields, "enabled")?.unwrap_or(true),
        })
    }

    fn parse_block_fields(&mut self) -> Result<BTreeMap<String, Value>> {
        self.expect_token(Token::LBrace, "{")?;
        let mut fields = BTreeMap::new();
        while !self.check(&Token::RBrace) {
            let key = self.expect_ident("field name")?;
            self.expect_token(Token::Colon, ":")?;
            let value = self.parse_value()?;
            fields.insert(key, value);
            if self.check(&Token::Comma) {
                self.index += 1;
            }
        }
        self.expect_token(Token::RBrace, "}")?;
        Ok(fields)
    }

    fn parse_value(&mut self) -> Result<Value> {
        match self.next_token() {
            Some(Token::String(value)) => Ok(Value::String(value)),
            Some(Token::Ident(value)) => Ok(Value::String(value)),
            Some(Token::Number(value)) => Ok(Value::Number(value)),
            Some(Token::Bool(value)) => Ok(Value::Bool(value)),
            Some(Token::LBracket) => {
                let mut items = Vec::new();
                while !self.check(&Token::RBracket) {
                    let value = match self.next_token() {
                        Some(Token::Ident(value)) | Some(Token::String(value)) => value,
                        other => bail!("期望列表项，但遇到 {other:?}"),
                    };
                    items.push(value);
                    if self.check(&Token::Comma) {
                        self.index += 1;
                    }
                }
                self.expect_token(Token::RBracket, "]")?;
                Ok(Value::List(items))
            }
            other => bail!("期望值，但遇到 {other:?}"),
        }
    }

    fn expect_ident(&mut self, label: &str) -> Result<String> {
        match self.next_token() {
            Some(Token::Ident(value)) => Ok(value),
            other => bail!("期望 {label}，但遇到 {other:?}"),
        }
    }

    fn expect_token(&mut self, expected: Token, label: &str) -> Result<()> {
        let token = self.next_token();
        if token == Some(expected) {
            Ok(())
        } else {
            bail!("期望令牌 {label}，但遇到 {token:?}")
        }
    }

    fn check(&self, expected: &Token) -> bool {
        self.tokens.get(self.index) == Some(expected)
    }

    fn next_token(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.index).cloned();
        if token.is_some() {
            self.index += 1;
        }
        token
    }

    fn is_eof(&self) -> bool {
        self.index >= self.tokens.len()
    }
}

fn field_number(fields: &BTreeMap<String, Value>, key: &str) -> Result<Option<f64>> {
    match fields.get(key) {
        None => Ok(None),
        Some(Value::Number(value)) => Ok(Some(*value)),
        other => bail!("字段 {key} 必须是数字，但遇到 {other:?}"),
    }
}

fn field_bool(fields: &BTreeMap<String, Value>, key: &str) -> Result<Option<bool>> {
    match fields.get(key) {
        None => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        other => bail!("字段 {key} 必须是布尔值，但遇到 {other:?}"),
    }
}

fn field_string(fields: &BTreeMap<String, Value>, key: &str) -> Result<Option<String>> {
    match fields.get(key) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        other => bail!("字段 {key} 必须是字符串或标识符，但遇到 {other:?}"),
    }
}

fn field_string_required(fields: &BTreeMap<String, Value>, key: &str) -> Result<String> {
    field_string(fields, key)?.ok_or_else(|| anyhow!("字段 {key} 是必需的"))
}

fn field_ident_required(fields: &BTreeMap<String, Value>, key: &str) -> Result<String> {
    field_string_required(fields, key)
}

fn field_list_required(fields: &BTreeMap<String, Value>, key: &str) -> Result<Vec<String>> {
    match fields.get(key) {
        Some(Value::List(items)) => Ok(items.clone()),
        None => bail!("字段 {key} 是必需的"),
        other => bail!("字段 {key} 必须是列表，但遇到 {other:?}"),
    }
}

fn parse_exchange(input: &str) -> Result<Exchange> {
    match input.to_ascii_lowercase().as_str() {
        "binance" => Ok(Exchange::Binance),
        "okx" => Ok(Exchange::Okx),
        other => bail!("不支持的交易所: {other}"),
    }
}

fn parse_symbol(input: &str) -> Result<Symbol> {
    match input.to_ascii_uppercase().as_str() {
        "BTCUSDT" => Ok(Symbol::BtcUsdt),
        other => bail!("不支持的交易对: {other}"),
    }
}

fn parse_market_type(input: &str) -> Result<MarketType> {
    match input.to_ascii_lowercase().as_str() {
        "spot" => Ok(MarketType::Spot),
        other => bail!("不支持的市场类型: {other}"),
    }
}

fn parse_data_kind(input: &str) -> Result<DataKind> {
    match input.to_ascii_lowercase().as_str() {
        "kline" | "klineseries" | "kline_series" => Ok(DataKind::KlineSeries),
        "quote" => Ok(DataKind::Quote),
        other => bail!("不支持的数据种类: {other}"),
    }
}

fn parse_intent_kind(input: &str) -> Result<IntentKind> {
    match input.to_ascii_lowercase().as_str() {
        "long_term_buy" | "longbuy" => Ok(IntentKind::LongTermBuy),
        "long_term_sell" | "longsell" => Ok(IntentKind::LongTermSell),
        "rsi" => Ok(IntentKind::Rsi),
        "macd" => Ok(IntentKind::Macd),
        "momentum" => Ok(IntentKind::Momentum),
        "zscore" | "z_score" => Ok(IntentKind::ZScore),
        "quote_observe" | "quoteobserve" => Ok(IntentKind::QuoteObserve),
        "sma_crossover" => Ok(IntentKind::SmaCrossover),
        other => bail!("不支持的意图种类: {other}"),
    }
}

fn looks_like_config(input: &str) -> bool {
    let trimmed = input.trim_start();
    ["runtime {", "data ", "intent ", "agent ", "risk "]
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
}

fn looks_like_script(input: &str) -> bool {
    input.lines().map(str::trim).any(|line| {
        line.starts_with("fn ")
            || line.starts_with("async fn ")
            || line.starts_with("import ")
            || line.starts_with("from ")
            || line.starts_with("let ")
            || line.starts_with("emit Intent(")
    })
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    const SAMPLE_SCRIPT: &str = r#"
runtime {
  initial_cash_balance: 100000
  taker_fee_bps: 10
  default_slippage_bps: 5
  total_cost_buffer_bps: 20
}

data binance_btc_150d_1d {
  exchange: binance
  symbol: BTCUSDT
  market_type: spot
  kind: kline
  days: 150
  interval: "1d"
}

data binance_btc_quote {
  exchange: binance
  symbol: BTCUSDT
  market_type: spot
  kind: quote
}

intent intent_long_buy {
  name: "Long Buy"
  kind: long_term_buy
  inputs: [binance_btc_150d_1d]
}

intent intent_binance_quote {
  name: "Binance Quote"
  kind: quote_observe
  inputs: [binance_btc_quote]
}

agent agent_long_term {
  name: "Long Term Agent"
  intents: [intent_long_buy]
}

risk risk_global {
  name: "Global Risk"
  agents: [agent_long_term]
  max_total_leverage: 3
  max_exchange_leverage: 3
  min_action_interval_ms: 100
}
"#;

    #[test]
    fn parses_quant_script_into_program() {
        let program = parse_quant_script(SAMPLE_SCRIPT).unwrap();
        assert_eq!(program.data_sources.len(), 2);
        assert_eq!(program.intents.len(), 2);
        assert_eq!(program.agents.len(), 1);
        assert_eq!(program.risks.len(), 1);
        assert_eq!(program.runtime.initial_cash_balance, 100_000.0);
    }

    #[test]
    fn compiles_quant_script_into_runtime_config() {
        let config = parse_quant_script_config(SAMPLE_SCRIPT).unwrap();
        assert_eq!(config.data_sources[0].data_id, "binance_btc_150d_1d");
        assert_eq!(config.intents[0].intent_id, "intent_long_buy");
        assert_eq!(config.risks[0].risk_id, "risk_global");
    }

    #[test]
    fn validates_with_existing_runtime_compiler() {
        let compiled = compile_quant_script(SAMPLE_SCRIPT).unwrap();
        assert_eq!(compiled.protocol_name, "quantpilot/minimal-sim/v1");
    }

    #[test]
    fn compiles_example_file() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = Path::new(manifest_dir).join("example.qs");
        let compiled = compile_quant_script_file(path).unwrap();
        assert_eq!(compiled.config.agents.len(), 2);
    }

    #[test]
    fn rejects_missing_required_field() {
        let err = parse_quant_script(
            r#"
            data only_data {
              exchange: binance
            }
            "#,
        )
        .and_then(|program| compile_program_to_config(&program))
        .unwrap_err();
        assert!(err.to_string().contains("symbol"));
    }

    #[test]
    fn detects_source_kind_for_formal_script() {
        let source = parse_quant_script_source(
            r#"
import math
fn strategy() {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#,
        )
        .unwrap();
        assert!(matches!(source, QuantScriptSource::Script(_)));
    }

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

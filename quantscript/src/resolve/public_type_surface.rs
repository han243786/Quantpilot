use crate::diagnostics::{Diagnostic, DiagnosticSeverity};
use crate::hir::TypedHirModule;
use crate::script::{Expr, Stmt};
use crate::types::{TypeArena, TypeId};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ResolveResult {
    pub module: TypedHirModule,
    pub types: TypeArena,
    pub diagnostics: Vec<Diagnostic>,
    pub expr_semantics: BTreeMap<String, ResolvedExprSemantic>,
    pub callables: BTreeMap<String, ResolvedCallable>,
    pub functions: BTreeMap<String, ResolvedFunction>,
}

impl ResolveResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedFunction {
    pub name: String,
    pub callable_kind: ResolvedCallableKind,
    pub param_names: Vec<String>,
    pub body: Vec<Stmt>,
    pub return_type: TypeId,
    pub return_expr: Option<Expr>,
    pub returned_list_target: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedCallableKind {
    BuiltinMath,
    FetchLike,
    Imported,
    UserFunction,
    ChangeHelper(ChangeHelperKind),
    IndicatorHelper(KnownIndicatorHelperKind),
    UniverseHelper(KnownUniverseHelperKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeHelperKind {
    Gain,
    Loss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownIndicatorHelperKind {
    MovingAverage(MovingAverageHelperKind),
    Rsi(RsiHelperKind),
    Macd,
    Momentum,
    ZScore,
    Atr,
    BollingerBands,
    Obv,
    Cmf,
    Adx,
    Stochastic,
    Cci,
    ParabolicSar,
    KeltnerChannel,
    DonchianChannel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownUniverseHelperKind {
    Symbols,
    Universe,
    Filter,
    SortBy,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovingAverageHelperKind {
    Sma,
    Ema,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RsiHelperKind {
    Wilder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedChangeSmoothingKind {
    Wilder,
    Ema,
    Simple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedFetchSourceKind {
    KlineSeries,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedSeriesCapabilityKind {
    Histogram,
    Boundary(ResolvedSeriesBoundaryKind),
    WindowAggregate(ResolvedWindowAggregateKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedSeriesBoundaryKind {
    First,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedWindowAggregateKind {
    Sum,
    Mean,
    StdDev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedBuiltinMathKind {
    Abs,
    Numeric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedMemberMutationKind {
    Push,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedExprSemantic {
    SeriesView(ResolvedSeriesViewKind),
    SeriesCapability(ResolvedSeriesCapabilityKind),
    WindowAggregateView(ResolvedWindowAggregateView),
    BoundaryLookbackPair {
        span: usize,
    },
    BalancedSmoothedChangePair {
        period: usize,
        smoothing: ResolvedChangeSmoothingKind,
    },
    ManualIndicatorFormula(ResolvedManualIndicatorFormula),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedSeriesViewKind {
    Current,
    First,
    Lookback(usize),
    Window(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedWindowAggregateView {
    pub aggregate_kind: ResolvedWindowAggregateKind,
    pub span: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedManualIndicatorFormula {
    Momentum {
        lookback: usize,
    },
    MovingAverage {
        span: usize,
    },
    MacdSignal {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    },
    MacdHistogram {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    },
    MacdLine {
        fast_period: usize,
        slow_period: usize,
    },
    ZScore {
        window: usize,
    },
}

#[derive(Debug, Clone)]
pub struct ResolvedCallable {
    pub name: String,
    pub kind: ResolvedCallableKind,
    pub change_smoothing_kind: Option<ResolvedChangeSmoothingKind>,
    pub fetch_source_kind: Option<ResolvedFetchSourceKind>,
    pub return_type: TypeId,
}

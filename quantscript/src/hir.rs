use crate::diagnostics::Span;
use crate::script::{BinaryOp, UnaryOp};
use crate::types::TypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExprId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub struct TypedHirModule {
    pub imports: Vec<HirImport>,
    pub functions: Vec<HirFunction>,
    pub test_blocks: Vec<HirTestBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirTestBlock {
    pub name: String,
    pub cover: Vec<String>,
    pub steps: Vec<HirStepBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirStepBlock {
    pub name: String,
    pub actions: Vec<HirTestAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirTestAction {
    Compile,
    Run {
        mode: String,
        duration_secs: u64,
        save: bool,
    },
    Backtest {
        source: String,
        start: Option<String>,
        end: Option<String>,
        seed: Option<u64>,
        save: bool,
    },
    Assert(String),
    SaveRun,
    Modify {
        node: String,
        param: String,
        value: HirTestParamValue,
    },
    Wait {
        condition: String,
        timeout_secs: u64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirTestParamValue {
    Number(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirImport {
    pub module: String,
    pub version: Option<String>,
    pub names: Vec<HirImportName>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirImportName {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirFunction {
    pub def_id: DefId,
    pub name: String,
    pub is_async: bool,
    pub params: Vec<HirParam>,
    pub return_type: TypeId,
    pub body: Vec<HirStmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirParam {
    pub def_id: DefId,
    pub name: String,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    Let(HirLetStmt),
    Return(Option<HirExpr>),
    EmitIntent {
        args: Vec<HirCallArg>,
        span: Span,
    },
    If {
        condition: HirExpr,
        then_branch: Vec<HirStmt>,
        else_if_branches: Vec<(HirExpr, Vec<HirStmt>)>,
        else_branch: Option<Vec<HirStmt>>,
        span: Span,
    },
    For {
        binding: HirBindingPattern,
        iterable: HirExpr,
        body: Vec<HirStmt>,
        span: Span,
    },
    While {
        condition: HirExpr,
        body: Vec<HirStmt>,
        span: Span,
    },
    Match {
        expr: HirExpr,
        arms: Vec<HirMatchArm>,
        span: Span,
    },
    Expr(HirExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirLetStmt {
    pub binding: HirBindingPattern,
    pub value: HirExpr,
    pub mutable: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirBindingPattern {
    pub def_id: DefId,
    pub name: String,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirCallArg {
    pub name: Option<String>,
    pub value: HirExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirMatchArm {
    pub pattern: String,
    pub body: HirMatchArmBody,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirMatchArmBody {
    Statement(Box<HirStmt>),
    Expr(HirExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirExpr {
    pub expr_id: ExprId,
    pub kind: HirExprKind,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirExprKind {
    Raw(String),
    Identifier(String),
    Number(f64),
    String(String),
    Bool(bool),
    List(Vec<HirExpr>),
    Call {
        callee: Box<HirExpr>,
        args: Vec<HirCallArg>,
    },
    Member {
        object: Box<HirExpr>,
        field: String,
    },
    Index {
        object: Box<HirExpr>,
        index: Box<HirExpr>,
    },
    Slice {
        object: Box<HirExpr>,
        start: Option<Box<HirExpr>>,
        end: Option<Box<HirExpr>>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<HirExpr>,
    },
    Binary {
        left: Box<HirExpr>,
        op: BinaryOp,
        right: Box<HirExpr>,
    },
    Range {
        start: Box<HirExpr>,
        end: Box<HirExpr>,
    },
    Await(Box<HirExpr>),
    Try(Box<HirExpr>),
}

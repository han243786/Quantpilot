use anyhow::{anyhow, bail, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptModule {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Import(ImportDecl),
    Function(FunctionDecl),
    TestBlock(TestBlock),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestBlock {
    pub name: String,
    pub cover: Vec<String>,
    pub steps: Vec<StepBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepBlock {
    pub name: String,
    pub actions: Vec<TestAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestAction {
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
        value: TestParamValue,
    },
    Wait {
        condition: String,
        timeout_secs: u64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestParamValue {
    Number(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub module: String,
    pub version: Option<String>,
    pub names: Option<Vec<ImportName>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportName {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub is_async: bool,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        pattern: String,
        ty: Option<String>,
        value: Expr,
        mutable: bool,
    },
    Return(Option<Expr>),
    EmitIntent {
        args: Vec<CallArg>,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_if_branches: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
    },
    For {
        pattern: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Match {
        expr: Expr,
        arms: Vec<MatchArm>,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallArg {
    pub name: Option<String>,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: String,
    pub body: MatchArmBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchArmBody {
    Statement(Box<Stmt>),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Raw(String),
    Identifier(String),
    Number(f64),
    String(String),
    Bool(bool),
    List(Vec<Expr>),
    Call {
        callee: Box<Expr>,
        args: Vec<CallArg>,
    },
    Member {
        object: Box<Expr>,
        field: String,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Slice {
        object: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },
    Await(Box<Expr>),
    Try(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,
    And,
    Or,
}

pub fn parse_quant_script_module(input: &str) -> Result<ScriptModule> {
    let lines = preprocess(input);
    let mut parser = ScriptParser { lines, index: 0 };
    parser.parse_module()
}

struct ScriptParser {
    lines: Vec<String>,
    index: usize,
}

impl ScriptParser {
    fn parse_module(&mut self) -> Result<ScriptModule> {
        let mut items = Vec::new();
        while let Some(line) = self.peek_line() {
            if line.starts_with("import ") {
                items.push(Item::Import(self.parse_import()?));
            } else if line.starts_with("from ") {
                items.push(Item::Import(self.parse_from_import()?));
            } else if line.starts_with("fn ") || line.starts_with("async fn ") {
                items.push(Item::Function(self.parse_function()?));
            } else if line.starts_with("@test ") || line.starts_with("@test{") {
                items.push(Item::TestBlock(self.parse_test_block()?));
            } else {
                bail!("unsupported top-level statement: {line}");
            }
        }
        Ok(ScriptModule { items })
    }

    fn parse_test_block(&mut self) -> Result<TestBlock> {
        let line = self.take_line()?;
        let _header = line
            .trim_start_matches("@test")
            .trim()
            .strip_suffix('{')
            .map(str::trim)
            .unwrap_or("");
        let fields = self.parse_test_fields()?;
        let name = fields
            .get("name")
            .cloned()
            .unwrap_or_else(|| "unnamed".to_string());
        let cover = fields
            .get("cover")
            .map(|v| {
                let inner = v.trim().trim_start_matches('[').trim_end_matches(']').trim();
                if inner.is_empty() {
                    return Vec::new();
                }
                // Count quotes to detect malformed arrays like ["A" "B"]
                let quote_count = inner.chars().filter(|&c| c == '"').count();
                if quote_count % 2 != 0 {
                    // Malformed — return empty rather than garbage
                    return Vec::new();
                }
                inner
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        let mut steps = Vec::new();
        while let Some(peeked) = self.peek_line() {
            if peeked.starts_with("@step(") {
                steps.push(self.parse_step_block()?);
            } else {
                break;
            }
        }
        Ok(TestBlock { name, cover, steps })
    }

    fn parse_step_block(&mut self) -> Result<StepBlock> {
        let line = self.take_line()?;
        const MAX_NAME_LEN: usize = 500;
        // Extract name from @step("name") or @step("name") {
        // Find content between first " and second "
        let mut name = line
            .split('"')
            .nth(1)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unnamed".to_string());
        if name.len() > MAX_NAME_LEN {
            name.truncate(MAX_NAME_LEN);
        }

        // If the line doesn't end with {, the opening brace may be on the next line
        if !line.trim_end().ends_with('{') {
            if self.index < self.lines.len() && self.lines[self.index].trim() == "{" {
                self.index += 1;
            }
        }

        let mut actions = Vec::new();
        while let Some(peeked) = self.peek_line() {
            let trimmed = peeked.trim();
            if trimmed == "}" {
                self.index += 1;
                break;
            }
            if trimmed.starts_with("@compile") {
                self.index += 1;
                actions.push(TestAction::Compile);
            } else if trimmed.starts_with("@assert ") {
                let expr = trimmed.trim_start_matches("@assert ").trim().to_string();
                self.index += 1;
                actions.push(TestAction::Assert(expr));
            } else if trimmed.starts_with("@save_run") {
                self.index += 1;
                actions.push(TestAction::SaveRun);
            } else if trimmed.starts_with("@run ") || trimmed.starts_with("@run{") {
                actions.push(self.parse_test_run_action()?);
            } else if trimmed.starts_with("@backtest ") || trimmed.starts_with("@backtest{") {
                actions.push(self.parse_test_backtest_action()?);
            } else if trimmed.starts_with("@modify ") {
                actions.push(self.parse_test_modify_action()?);
            } else if trimmed.starts_with("@wait ") {
                actions.push(self.parse_test_wait_action()?);
            } else {
                self.index += 1;
            }
        }
        Ok(StepBlock { name, actions })
    }

    fn parse_test_run_action(&mut self) -> Result<TestAction> {
        let line = self.take_line()?;
        let fields = if line.contains('{') {
            let inner = line
                .trim_start_matches("@run")
                .trim()
                .trim_start_matches('{')
                .trim_end_matches('}')
                .trim();
            parse_test_inline_fields(inner)
        } else {
            // multi-line: @run { \n ... \n }
            self.parse_test_action_fields()?
        };
        Ok(TestAction::Run {
            mode: fields.get("mode").cloned().unwrap_or_else(|| "paper".to_string()),
            duration_secs: fields
                .get("duration")
                .and_then(|v| parse_duration_secs(v))
                .unwrap_or(0), // 0 triggers validation error in runner
            save: fields
                .get("save")
                .map(|v| v == "true")
                .unwrap_or(false),
        })
    }

    fn parse_test_backtest_action(&mut self) -> Result<TestAction> {
        let line = self.take_line()?;
        let fields = if line.contains('{') {
            let inner = line
                .trim_start_matches("@backtest")
                .trim()
                .trim_start_matches('{')
                .trim_end_matches('}')
                .trim();
            parse_test_inline_fields(inner)
        } else {
            self.parse_test_action_fields()?
        };
        Ok(TestAction::Backtest {
            source: fields
                .get("source")
                .cloned()
                .unwrap_or_else(|| "deterministic_mock".to_string()),
            start: fields.get("start").cloned(),
            end: fields.get("end").cloned(),
            seed: fields.get("seed").and_then(|v| v.parse::<u64>().ok()),
            save: fields
                .get("save")
                .map(|v| v == "true")
                .unwrap_or(false),
        })
    }

    fn parse_test_modify_action(&mut self) -> Result<TestAction> {
        let line = self.take_line()?;
        let stripped = line.trim_start_matches("@modify").trim();
        let fields = if stripped.starts_with('{') {
            let inner = stripped
                .trim_start_matches('{')
                .trim_end_matches('}')
                .trim();
            parse_test_inline_fields(inner)
        } else {
            parse_test_inline_fields(stripped)
        };
        let node = fields
            .get("node")
            .cloned()
            .unwrap_or_default();
        let param = fields
            .get("param")
            .cloned()
            .unwrap_or_default();
        let value = fields
            .get("value")
            .map(|v| {
                if let Ok(n) = v.parse::<f64>() {
                    TestParamValue::Number(n)
                } else if v == "true" || v == "false" {
                    TestParamValue::Bool(v == "true")
                } else {
                    TestParamValue::String(v.clone())
                }
            })
            .unwrap_or(TestParamValue::Number(0.0));
        Ok(TestAction::Modify { node, param, value })
    }

    fn parse_test_wait_action(&mut self) -> Result<TestAction> {
        let line = self.take_line()?;
        let stripped = line.trim_start_matches("@wait").trim();
        let fields = if stripped.starts_with('{') {
            let inner = stripped
                .trim_start_matches('{')
                .trim_end_matches('}')
                .trim();
            parse_test_inline_fields(inner)
        } else {
            parse_test_inline_fields(stripped)
        };
        Ok(TestAction::Wait {
            condition: fields
                .get("condition")
                .cloned()
                .unwrap_or_default(),
            timeout_secs: fields
                .get("timeout")
                .and_then(|v| parse_duration_secs(v))
                .unwrap_or(120),
        })
    }

    fn parse_test_fields(&mut self) -> Result<BTreeMap<String, String>> {
        let mut fields = BTreeMap::new();
        while self.index < self.lines.len() {
            let trimmed = self.lines[self.index].trim().to_string();
            if trimmed == "}" {
                self.index += 1;
                break;
            }
            if trimmed.starts_with('@') {
                break;
            }
            self.index += 1;
            if let Some((key, value)) = trimmed.split_once(':') {
                let val = value.trim().trim_end_matches(',').trim().trim_matches('"').trim().to_string();
                fields.insert(key.trim().to_string(), val);
            }
        }
        Ok(fields)
    }

    fn parse_test_action_fields(&mut self) -> Result<BTreeMap<String, String>> {
        let mut fields = BTreeMap::new();
        // Consume opening brace if present
        if self.index < self.lines.len() {
            if self.lines[self.index].trim() == "{" {
                self.index += 1;
            }
        }
        while self.index < self.lines.len() {
            let trimmed = self.lines[self.index].trim().to_string();
            if trimmed == "}" {
                self.index += 1;
                break;
            }
            self.index += 1;
            if let Some((key, value)) = trimmed.split_once(':') {
                let val = value.trim().trim_end_matches(',').trim().trim_matches('"').trim().to_string();
                fields.insert(key.trim().to_string(), val);
            }
        }
        Ok(fields)
    }

    fn parse_import(&mut self) -> Result<ImportDecl> {
        let line = self.take_line()?;
        Ok(ImportDecl {
            module: line.trim_start_matches("import ").trim().to_string(),
            version: None,
            names: None,
        })
    }

    fn parse_from_import(&mut self) -> Result<ImportDecl> {
        let line = self.take_line()?;
        let rest = line.trim_start_matches("from ").trim();
        let (module_part, names_part) = rest
            .split_once(" import ")
            .ok_or_else(|| anyhow!("invalid from-import syntax: {line}"))?;
        let (module, version) = module_part
            .split_once('@')
            .map(|(m, v)| (m.to_string(), Some(v.to_string())))
            .unwrap_or_else(|| (module_part.to_string(), None));
        let names = names_part
            .split(',')
            .map(|item| {
                let entry = item.trim();
                let (name, alias) = entry
                    .split_once(" as ")
                    .map(|(name, alias)| (name.trim().to_string(), Some(alias.trim().to_string())))
                    .unwrap_or_else(|| (entry.to_string(), None));
                ImportName { name, alias }
            })
            .collect();
        Ok(ImportDecl {
            module,
            version,
            names: Some(names),
        })
    }

    fn parse_function(&mut self) -> Result<FunctionDecl> {
        let line = self.take_line()?;
        let is_async = line.starts_with("async fn ");
        let signature = if is_async {
            line.trim_start_matches("async ").trim()
        } else {
            line.as_str()
        };
        let signature = signature
            .strip_suffix('{')
            .map(str::trim_end)
            .ok_or_else(|| anyhow!("function must open with '{{': {line}"))?;
        let rest = signature.trim_start_matches("fn ").trim();
        let name_end = rest
            .find('(')
            .ok_or_else(|| anyhow!("invalid function signature: {line}"))?;
        let name = rest[..name_end].trim().to_string();
        let after_name = &rest[name_end + 1..];
        let params_end = after_name
            .find(')')
            .ok_or_else(|| anyhow!("invalid function parameters: {line}"))?;
        let params_text = &after_name[..params_end];
        let after_params = after_name[params_end + 1..].trim();
        let return_type = after_params
            .strip_prefix("->")
            .map(|value| value.trim().to_string());
        let body = self.parse_block()?;
        Ok(FunctionDecl {
            is_async,
            name,
            params: parse_params(params_text),
            return_type,
            body,
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while let Some(line) = self.peek_line() {
            if line == "}" {
                self.index += 1;
                break;
            }
            statements.push(self.parse_stmt()?);
        }
        Ok(statements)
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        let line = self.take_line()?;
        if line.starts_with("let ") {
            return parse_let_stmt(&line);
        }
        if line.starts_with("return") {
            let value = line
                .trim_start_matches("return")
                .trim()
                .trim_end_matches(',')
                .trim();
            return Ok(Stmt::Return(
                (!value.is_empty()).then(|| parse_expr_lossy(value)),
            ));
        }
        if line.starts_with("emit Intent(") {
            let inside = line
                .trim_start_matches("emit Intent(")
                .trim_end_matches(')')
                .trim();
            return Ok(Stmt::EmitIntent {
                args: split_args(inside)
                    .into_iter()
                    .map(parse_call_arg)
                    .collect::<Result<Vec<_>>>()?,
            });
        }
        if line.starts_with("if ") {
            return self.parse_if(line);
        }
        if line.starts_with("for ") {
            return self.parse_for(line);
        }
        if line.starts_with("while ") {
            return self.parse_while(line);
        }
        if line.starts_with("match ") {
            return self.parse_match(line);
        }
        Ok(Stmt::Expr(parse_expr_lossy(line.trim_end_matches(','))))
    }

    fn parse_if(&mut self, line: String) -> Result<Stmt> {
        let condition = line
            .trim_start_matches("if ")
            .trim()
            .strip_suffix('{')
            .map(str::trim_end)
            .ok_or_else(|| anyhow!("if statement must open with '{{': {line}"))?;
        let then_branch = self.parse_block()?;
        let mut else_if_branches = Vec::new();
        let mut else_branch = None;

        loop {
            let Some(next) = self.peek_line() else { break };
            if next.starts_with("else if ") {
                let line = self.take_line()?;
                let condition = line
                    .trim_start_matches("else if ")
                    .trim()
                    .strip_suffix('{')
                    .map(str::trim_end)
                    .ok_or_else(|| anyhow!("else if must open with '{{': {line}"))?;
                else_if_branches.push((parse_expr_lossy(condition), self.parse_block()?));
                continue;
            }
            if next == "else {" {
                self.index += 1;
                else_branch = Some(self.parse_block()?);
            }
            break;
        }

        Ok(Stmt::If {
            condition: parse_expr_lossy(condition),
            then_branch,
            else_if_branches,
            else_branch,
        })
    }

    fn parse_for(&mut self, line: String) -> Result<Stmt> {
        let header = line
            .trim_start_matches("for ")
            .trim()
            .strip_suffix('{')
            .map(str::trim_end)
            .ok_or_else(|| anyhow!("for statement must open with '{{': {line}"))?;
        let (pattern, iterable) = header
            .split_once(" in ")
            .ok_or_else(|| anyhow!("invalid for syntax: {line}"))?;
        Ok(Stmt::For {
            pattern: pattern.trim().to_string(),
            iterable: parse_expr_lossy(iterable.trim()),
            body: self.parse_block()?,
        })
    }

    fn parse_while(&mut self, line: String) -> Result<Stmt> {
        let condition = line
            .trim_start_matches("while ")
            .trim()
            .strip_suffix('{')
            .map(str::trim_end)
            .ok_or_else(|| anyhow!("while statement must open with '{{': {line}"))?;
        Ok(Stmt::While {
            condition: parse_expr_lossy(condition),
            body: self.parse_block()?,
        })
    }

    fn parse_match(&mut self, line: String) -> Result<Stmt> {
        let expr = line
            .trim_start_matches("match ")
            .trim()
            .strip_suffix('{')
            .map(str::trim_end)
            .ok_or_else(|| anyhow!("match statement must open with '{{': {line}"))?;
        let mut arms = Vec::new();
        while let Some(next) = self.peek_line() {
            if next == "}" {
                self.index += 1;
                break;
            }
            let line = self.take_line()?;
            let (pattern, body) = line
                .split_once("=>")
                .ok_or_else(|| anyhow!("invalid match arm: {line}"))?;
            arms.push(MatchArm {
                pattern: pattern.trim().to_string(),
                body: parse_match_arm_body(body.trim().trim_end_matches(',').trim())?,
            });
        }
        Ok(Stmt::Match {
            expr: parse_expr_lossy(expr),
            arms,
        })
    }

    fn peek_line(&self) -> Option<&str> {
        self.lines.get(self.index).map(String::as_str)
    }

    fn take_line(&mut self) -> Result<String> {
        let line = self
            .lines
            .get(self.index)
            .cloned()
            .ok_or_else(|| anyhow!("unexpected end of input"))?;
        self.index += 1;
        Ok(line)
    }
}

fn preprocess(input: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in input.lines() {
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        for segment in line
            .replace("} else if ", "}\nelse if ")
            .replace("} else {", "}\nelse {")
            .split('\n')
        {
            let segment = segment.trim();
            if !segment.is_empty() {
                lines.push(segment.to_string());
            }
        }
    }
    lines
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut prev_escape = false;
    for (idx, ch) in line.char_indices() {
        if ch == '"' && !prev_escape {
            in_string = !in_string;
        }
        if ch == '#' && !in_string {
            return &line[..idx];
        }
        prev_escape = ch == '\\' && !prev_escape;
        if ch != '\\' {
            prev_escape = false;
        }
    }
    line
}

fn parse_params(input: &str) -> Vec<Param> {
    split_args(input)
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .map(|part| {
            let (name, ty) = part
                .split_once(':')
                .map(|(name, ty)| (name.trim().to_string(), Some(ty.trim().to_string())))
                .unwrap_or_else(|| (part.trim().to_string(), None));
            Param { name, ty }
        })
        .collect()
}

fn parse_let_stmt(line: &str) -> Result<Stmt> {
    let rhs = line.trim_start_matches("let ").trim();
    let (mutable, rhs) = if let Some(rest) = rhs.strip_prefix("mut ") {
        (true, rest.trim())
    } else {
        (false, rhs)
    };
    let (binding, value) = rhs
        .split_once('=')
        .ok_or_else(|| anyhow!("invalid let statement: {line}"))?;
    let (pattern, ty) = binding
        .split_once(':')
        .map(|(pattern, ty)| (pattern.trim().to_string(), Some(ty.trim().to_string())))
        .unwrap_or_else(|| (binding.trim().to_string(), None));
    Ok(Stmt::Let {
        pattern,
        ty,
        value: parse_expr_lossy(value.trim()),
        mutable,
    })
}

fn split_args(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0_i32;
    let mut bracket_depth = 0_i32;
    let mut brace_depth = 0_i32;
    let mut in_string = false;
    let mut prev_escape = false;

    for ch in input.chars() {
        match ch {
            '"' if !prev_escape => {
                in_string = !in_string;
                current.push(ch);
            }
            '(' if !in_string => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' if !in_string => {
                paren_depth -= 1;
                current.push(ch);
            }
            '[' if !in_string => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' if !in_string => {
                bracket_depth -= 1;
                current.push(ch);
            }
            '{' if !in_string => {
                brace_depth += 1;
                current.push(ch);
            }
            '}' if !in_string => {
                brace_depth -= 1;
                current.push(ch);
            }
            ',' if !in_string && paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
        prev_escape = ch == '\\' && !prev_escape;
        if ch != '\\' {
            prev_escape = false;
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        parts.push(trimmed.to_string());
    }
    parts
}

fn parse_call_arg(input: String) -> Result<CallArg> {
    if let Some((name, value)) = input.split_once(':').or_else(|| input.split_once('=')) {
        Ok(CallArg {
            name: Some(name.trim().to_string()),
            value: parse_expr_lossy(value.trim()),
        })
    } else {
        Ok(CallArg {
            name: None,
            value: parse_expr_lossy(input.trim()),
        })
    }
}

fn parse_match_arm_body(input: &str) -> Result<MatchArmBody> {
    if input.starts_with("emit Intent(") {
        Ok(MatchArmBody::Statement(Box::new(
            ScriptParser {
                lines: vec![input.to_string()],
                index: 0,
            }
            .parse_stmt()?,
        )))
    } else {
        Ok(MatchArmBody::Expr(parse_expr_lossy(input)))
    }
}

fn parse_test_inline_fields(input: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    for part in split_args(input) {
        let trimmed = part.trim();
        if let Some((key, value)) = trimmed.split_once(':') {
            let val = value.trim().trim_matches('"').trim().to_string();
            fields.insert(key.trim().to_string(), val);
        }
    }
    fields
}

fn parse_duration_secs(input: &str) -> Option<u64> {
    let input = input.trim().trim_matches('"');
    if let Ok(n) = input.parse::<u64>() {
        return Some(n);
    }
    // Parse "60s", "2m", "5min" format
    if let Some(rest) = input.strip_suffix('s') {
        return rest.trim().parse::<u64>().ok();
    }
    if let Some(rest) = input.strip_suffix("min") {
        return rest.trim().parse::<u64>().ok().map(|v| v * 60);
    }
    if let Some(rest) = input.strip_suffix('m') {
        return rest.trim().parse::<u64>().ok().map(|v| v * 60);
    }
    None
}

pub fn parse_expr(input: &str) -> Result<Expr> {
    let tokens = tokenize_expr(input)?;
    let mut parser = ExprParser { tokens, index: 0 };
    let expr = parser.parse_expression(0)?;
    if !parser.is_eof() {
        bail!("unexpected trailing expression tokens in: {input}");
    }
    Ok(expr)
}

fn parse_expr_lossy(input: &str) -> Expr {
    parse_expr(input).unwrap_or_else(|_| Expr::Raw(input.trim().to_string()))
}

#[derive(Debug, Clone, PartialEq)]
enum ExprToken {
    Ident(String),
    String(String),
    Number(f64),
    Bool(bool),
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Colon,
    Equal,
    Question,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    EqualEqual,
    NotEqual,
    AndAnd,
    OrOr,
    DotDot,
}

fn tokenize_expr(input: &str) -> Result<Vec<ExprToken>> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            c if c.is_whitespace() => {
                chars.next();
            }
            '"' => {
                chars.next();
                let mut content = String::new();
                let mut escaped = false;
                for next in chars.by_ref() {
                    if escaped {
                        content.push(match next {
                            'n' => '\n',
                            't' => '\t',
                            '"' => '"',
                            '\\' => '\\',
                            other => other,
                        });
                        escaped = false;
                    } else if next == '\\' {
                        escaped = true;
                    } else if next == '"' {
                        break;
                    } else {
                        content.push(next);
                    }
                }
                tokens.push(ExprToken::String(content));
            }
            '\'' => {
                chars.next();
                let mut content = String::new();
                for next in chars.by_ref() {
                    if next == '\'' {
                        break;
                    }
                    content.push(next);
                }
                tokens.push(ExprToken::String(content));
            }
            '(' => {
                chars.next();
                tokens.push(ExprToken::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(ExprToken::RParen);
            }
            '[' => {
                chars.next();
                tokens.push(ExprToken::LBracket);
            }
            ']' => {
                chars.next();
                tokens.push(ExprToken::RBracket);
            }
            ',' => {
                chars.next();
                tokens.push(ExprToken::Comma);
            }
            ':' => {
                chars.next();
                tokens.push(ExprToken::Colon);
            }
            '?' => {
                chars.next();
                tokens.push(ExprToken::Question);
            }
            '+' => {
                chars.next();
                tokens.push(ExprToken::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(ExprToken::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(ExprToken::Star);
            }
            '/' => {
                chars.next();
                tokens.push(ExprToken::Slash);
            }
            '%' => {
                chars.next();
                tokens.push(ExprToken::Percent);
            }
            '.' => {
                chars.next();
                if matches!(chars.peek(), Some('.')) {
                    chars.next();
                    tokens.push(ExprToken::DotDot);
                } else {
                    tokens.push(ExprToken::Dot);
                }
            }
            '!' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(ExprToken::NotEqual);
                } else {
                    tokens.push(ExprToken::Bang);
                }
            }
            '=' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(ExprToken::EqualEqual);
                } else {
                    tokens.push(ExprToken::Equal);
                }
            }
            '>' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(ExprToken::GreaterEqual);
                } else {
                    tokens.push(ExprToken::Greater);
                }
            }
            '<' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(ExprToken::LessEqual);
                } else {
                    tokens.push(ExprToken::Less);
                }
            }
            '&' => {
                chars.next();
                if matches!(chars.peek(), Some('&')) {
                    chars.next();
                    tokens.push(ExprToken::AndAnd);
                } else {
                    bail!("unsupported '&' in expression: {input}");
                }
            }
            '|' => {
                chars.next();
                if matches!(chars.peek(), Some('|')) {
                    chars.next();
                    tokens.push(ExprToken::OrOr);
                } else {
                    bail!("unsupported '|' in expression: {input}");
                }
            }
            c if c.is_ascii_digit() => {
                let mut number = String::new();
                let mut saw_decimal = false;
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        number.push(next);
                        chars.next();
                    } else if next == '.'
                        && !saw_decimal
                        && !matches!(chars.clone().nth(1), Some('.'))
                    {
                        saw_decimal = true;
                        number.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(ExprToken::Number(number.parse::<f64>()?));
            }
            _ => {
                let mut ident = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_alphanumeric() || matches!(next, '_') {
                        ident.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if ident.is_empty() {
                    bail!("unexpected character in expression: {ch}");
                }
                match ident.as_str() {
                    "true" => tokens.push(ExprToken::Bool(true)),
                    "false" => tokens.push(ExprToken::Bool(false)),
                    "and" => tokens.push(ExprToken::AndAnd),
                    "or" => tokens.push(ExprToken::OrOr),
                    "not" => tokens.push(ExprToken::Bang),
                    _ => tokens.push(ExprToken::Ident(ident)),
                }
            }
        }
    }

    Ok(tokens)
}

struct ExprParser {
    tokens: Vec<ExprToken>,
    index: usize,
}

impl ExprParser {
    fn parse_expression(&mut self, min_prec: u8) -> Result<Expr> {
        let mut left = self.parse_prefix()?;

        loop {
            if let Some(postfix) = self.peek() {
                match postfix {
                    ExprToken::LParen => {
                        left = self.parse_call(left)?;
                        continue;
                    }
                    ExprToken::Dot => {
                        self.index += 1;
                        let field = self.expect_ident("field name")?;
                        left = Expr::Member {
                            object: Box::new(left),
                            field,
                        };
                        continue;
                    }
                    ExprToken::LBracket => {
                        left = self.parse_index_or_slice(left)?;
                        continue;
                    }
                    ExprToken::Question => {
                        self.index += 1;
                        left = Expr::Try(Box::new(left));
                        continue;
                    }
                    _ => {}
                }
            }

            if matches!(self.peek(), Some(ExprToken::DotDot)) {
                if min_prec > 0 {
                    break;
                }
                self.index += 1;
                let right = self.parse_expression(0)?;
                left = Expr::Range {
                    start: Box::new(left),
                    end: Box::new(right),
                };
                continue;
            }

            let Some((op, prec, right_assoc)) = self.peek_binary_op() else {
                break;
            };
            if prec < min_prec {
                break;
            }
            self.index += 1;
            let next_min = if right_assoc { prec } else { prec + 1 };
            let right = self.parse_expression(next_min)?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr> {
        match self.next() {
            Some(ExprToken::Ident(value)) if value == "await" => {
                Ok(Expr::Await(Box::new(self.parse_expression(8)?)))
            }
            Some(ExprToken::Ident(value)) => Ok(Expr::Identifier(value)),
            Some(ExprToken::String(value)) => Ok(Expr::String(value)),
            Some(ExprToken::Number(value)) => Ok(Expr::Number(value)),
            Some(ExprToken::Bool(value)) => Ok(Expr::Bool(value)),
            Some(ExprToken::Minus) => Ok(Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(self.parse_expression(8)?),
            }),
            Some(ExprToken::Bang) => Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(self.parse_expression(8)?),
            }),
            Some(ExprToken::LParen) => {
                let expr = self.parse_expression(0)?;
                self.expect_token(ExprToken::RParen, ")")?;
                Ok(expr)
            }
            Some(ExprToken::LBracket) => {
                let mut items = Vec::new();
                while !self.check(&ExprToken::RBracket) {
                    items.push(self.parse_expression(0)?);
                    if self.check(&ExprToken::Comma) {
                        self.index += 1;
                    }
                }
                self.expect_token(ExprToken::RBracket, "]")?;
                Ok(Expr::List(items))
            }
            other => bail!("expected expression, found {other:?}"),
        }
    }

    fn parse_call(&mut self, callee: Expr) -> Result<Expr> {
        self.expect_token(ExprToken::LParen, "(")?;
        let mut args = Vec::new();
        while !self.check(&ExprToken::RParen) {
            args.push(self.parse_call_arg_expr()?);
            if self.check(&ExprToken::Comma) {
                self.index += 1;
            }
        }
        self.expect_token(ExprToken::RParen, ")")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
        })
    }

    fn parse_call_arg_expr(&mut self) -> Result<CallArg> {
        let save = self.index;
        if let Some(ExprToken::Ident(name)) = self.next() {
            if self.check(&ExprToken::Colon) || self.check(&ExprToken::Equal) {
                self.index = save;
            } else if let Some(token) = self.peek() {
                if matches!(token, ExprToken::Comma | ExprToken::RParen) {
                    return Ok(CallArg {
                        name: None,
                        value: Expr::Identifier(name),
                    });
                }
                self.index = save;
            } else {
                self.index = save;
            }
        } else {
            self.index = save;
        }

        if let Some(ExprToken::Ident(name)) = self.peek().cloned() {
            if matches!(
                self.tokens.get(self.index + 1),
                Some(ExprToken::Colon) | Some(ExprToken::Equal)
            ) {
                self.index += 2;
                return Ok(CallArg {
                    name: Some(name),
                    value: self.parse_expression(0)?,
                });
            }
        }

        Ok(CallArg {
            name: None,
            value: self.parse_expression(0)?,
        })
    }

    fn parse_index_or_slice(&mut self, object: Expr) -> Result<Expr> {
        self.expect_token(ExprToken::LBracket, "[")?;
        if self.check(&ExprToken::DotDot) {
            self.index += 1;
            let end = if self.check(&ExprToken::RBracket) {
                None
            } else {
                Some(Box::new(self.parse_expression(0)?))
            };
            self.expect_token(ExprToken::RBracket, "]")?;
            return Ok(Expr::Slice {
                object: Box::new(object),
                start: None,
                end,
            });
        }

        let first = self.parse_expression(1)?;
        if self.check(&ExprToken::DotDot) {
            self.index += 1;
            let end = if self.check(&ExprToken::RBracket) {
                None
            } else {
                Some(Box::new(self.parse_expression(0)?))
            };
            self.expect_token(ExprToken::RBracket, "]")?;
            Ok(Expr::Slice {
                object: Box::new(object),
                start: Some(Box::new(first)),
                end,
            })
        } else {
            self.expect_token(ExprToken::RBracket, "]")?;
            Ok(Expr::Index {
                object: Box::new(object),
                index: Box::new(first),
            })
        }
    }

    fn peek_binary_op(&self) -> Option<(BinaryOp, u8, bool)> {
        match self.peek()? {
            ExprToken::OrOr => Some((BinaryOp::Or, 1, false)),
            ExprToken::AndAnd => Some((BinaryOp::And, 2, false)),
            ExprToken::EqualEqual => Some((BinaryOp::Equal, 3, false)),
            ExprToken::NotEqual => Some((BinaryOp::NotEqual, 3, false)),
            ExprToken::Greater => Some((BinaryOp::Greater, 4, false)),
            ExprToken::GreaterEqual => Some((BinaryOp::GreaterEqual, 4, false)),
            ExprToken::Less => Some((BinaryOp::Less, 4, false)),
            ExprToken::LessEqual => Some((BinaryOp::LessEqual, 4, false)),
            ExprToken::Plus => Some((BinaryOp::Add, 5, false)),
            ExprToken::Minus => Some((BinaryOp::Subtract, 5, false)),
            ExprToken::Star => Some((BinaryOp::Multiply, 6, false)),
            ExprToken::Slash => Some((BinaryOp::Divide, 6, false)),
            ExprToken::Percent => Some((BinaryOp::Modulo, 6, false)),
            _ => None,
        }
    }

    fn expect_ident(&mut self, label: &str) -> Result<String> {
        match self.next() {
            Some(ExprToken::Ident(value)) => Ok(value),
            other => bail!("expected {label}, found {other:?}"),
        }
    }

    fn expect_token(&mut self, expected: ExprToken, label: &str) -> Result<()> {
        let token = self.next();
        if token == Some(expected) {
            Ok(())
        } else {
            bail!("expected token {label}, found {token:?}")
        }
    }

    fn check(&self, expected: &ExprToken) -> bool {
        self.tokens.get(self.index) == Some(expected)
    }

    fn peek(&self) -> Option<&ExprToken> {
        self.tokens.get(self.index)
    }

    fn next(&mut self) -> Option<ExprToken> {
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

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC_STYLE_SCRIPT: &str = r#"
import math
from data import fetch as get_data
from signals@1.2 import rsi, macd

fn moving_average(series: Series<Float>, period: Int) -> Float {
    let n = period
    return series[n..].sum() / n
}

async fn download_all(symbols: List<String>) -> List<IntervalK> {
    let mut tasks = []
    for s in symbols {
        tasks.push(spawn async { fetch(s, interval="1m", lookback=100)? })
    }
    return await all(tasks)
}

fn strategy() {
    let data = get_data("BTCUSDT")
    if data.ok() {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else if data.retryable() {
        log_warn("retry")
    } else {
        log_error("failed")
    }

    match read_data("BTCUSDT") {
        Ok(k) => emit Intent("BUY", instrument="BTCUSDT", quantity=1.0),
        Err(e) => log_error(e)
    }
}
"#;

    #[test]
    fn parser_accepts_legacy_import_and_function_surface_for_compatibility_only() {
        // Parser-only compatibility coverage; this is not product authoring support.
        let module = parse_quant_script_module(SPEC_STYLE_SCRIPT).unwrap();
        assert_eq!(module.items.len(), 6);
    }

    #[test]
    fn parser_accepts_legacy_if_and_match_syntax_for_compatibility_only() {
        // Parser-only compatibility coverage; runtime lowering owns support decisions.
        let module = parse_quant_script_module(SPEC_STYLE_SCRIPT).unwrap();
        let strategy = module
            .items
            .iter()
            .find_map(|item| match item {
                Item::Function(function) if function.name == "strategy" => Some(function),
                _ => None,
            })
            .unwrap();
        assert!(strategy
            .body
            .iter()
            .any(|stmt| matches!(stmt, Stmt::If { .. })));
        assert!(strategy
            .body
            .iter()
            .any(|stmt| matches!(stmt, Stmt::Match { .. })));
    }

    #[test]
    fn parses_emit_intent_named_and_positional_args() {
        let module = parse_quant_script_module(
            r#"
fn strategy() {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#,
        )
        .unwrap();
        let function = match &module.items[0] {
            Item::Function(function) => function,
            _ => panic!("expected function"),
        };
        let stmt = match &function.body[0] {
            Stmt::EmitIntent { args } => args,
            _ => panic!("expected emit intent"),
        };
        assert_eq!(stmt.len(), 3);
        assert_eq!(stmt[0].name, None);
        assert_eq!(stmt[1].name.as_deref(), Some("instrument"));
    }

    #[test]
    fn parses_numeric_negative_slice_bounds() {
        let expr = parse_expr("values[-3..]").unwrap();
        let Expr::Slice { start, end, .. } = expr else {
            panic!("expected slice expression");
        };
        assert!(end.is_none());
        assert!(matches!(
            start.as_deref(),
            Some(Expr::Unary {
                op: UnaryOp::Negate,
                ..
            })
        ));
    }
}

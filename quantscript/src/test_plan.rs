use crate::script::{ScriptModule, Item, StepBlock, TestAction, TestParamValue};

#[derive(Debug, Clone, PartialEq)]
pub struct TestPlan {
    pub scenario_name: String,
    pub cover: Vec<String>,
    pub steps: Vec<TestStep>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestStep {
    pub name: String,
    pub actions: Vec<TestActionDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestActionDef {
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
        value: TestParamValueDef,
    },
    Wait {
        condition: String,
        timeout_secs: u64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestParamValueDef {
    Number(f64),
    String(String),
    Bool(bool),
}

impl From<&TestParamValue> for TestParamValueDef {
    fn from(value: &TestParamValue) -> Self {
        match value {
            TestParamValue::Number(n) => TestParamValueDef::Number(*n),
            TestParamValue::String(s) => TestParamValueDef::String(s.clone()),
            TestParamValue::Bool(b) => TestParamValueDef::Bool(*b),
        }
    }
}

impl From<&TestAction> for TestActionDef {
    fn from(action: &TestAction) -> Self {
        match action {
            TestAction::Compile => TestActionDef::Compile,
            TestAction::Run { mode, duration_secs, save } => TestActionDef::Run {
                mode: mode.clone(),
                duration_secs: *duration_secs,
                save: *save,
            },
            TestAction::Backtest { source, start, end, seed, save } => TestActionDef::Backtest {
                source: source.clone(),
                start: start.clone(),
                end: end.clone(),
                seed: *seed,
                save: *save,
            },
            TestAction::Assert(expr) => TestActionDef::Assert(expr.clone()),
            TestAction::SaveRun => TestActionDef::SaveRun,
            TestAction::Modify { node, param, value } => TestActionDef::Modify {
                node: node.clone(),
                param: param.clone(),
                value: value.into(),
            },
            TestAction::Wait { condition, timeout_secs } => TestActionDef::Wait {
                condition: condition.clone(),
                timeout_secs: *timeout_secs,
            },
        }
    }
}

impl From<&StepBlock> for TestStep {
    fn from(step: &StepBlock) -> Self {
        TestStep {
            name: step.name.clone(),
            actions: step.actions.iter().map(TestActionDef::from).collect(),
        }
    }
}

pub fn extract_test_plan(module: &ScriptModule) -> Option<TestPlan> {
    for item in &module.items {
        if let Item::TestBlock(test_block) = item {
            return Some(TestPlan {
                scenario_name: test_block.name.clone(),
                cover: test_block.cover.clone(),
                steps: test_block.steps.iter().map(TestStep::from).collect(),
            });
        }
    }
    None
}

/// Extract test blocks while also filtering out test items to leave pure strategy
pub fn split_test_items(module: &ScriptModule) -> (ScriptModule, Vec<TestPlan>) {
    let mut strategy_items = Vec::new();
    let mut test_plans = Vec::new();

    for item in &module.items {
        match item {
            Item::TestBlock(test_block) => {
                test_plans.push(TestPlan {
                    scenario_name: test_block.name.clone(),
                    cover: test_block.cover.clone(),
                    steps: test_block.steps.iter().map(TestStep::from).collect(),
                });
            }
            other => strategy_items.push(other.clone()),
        }
    }

    (ScriptModule { items: strategy_items }, test_plans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::parse_quant_script_module;

    #[test]
    fn extracts_test_plan_from_module() {
        let source = r#"
fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=300)?
    let fast = sma(closes, 20)
    let slow = sma(closes, 50)
    if fast > slow {
        emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
    } else {
        emit Intent("SELL", instrument="BTCUSDT", quantity=1.0)
    }
}

@test {
    name: "场景一：BTC双均线"
    cover: ["P-03", "STRAT-001"]
}

@step("编译策略") {
    @compile
    @assert compile.compilable == true
}

@step("Paper运行") {
    @run { mode: "paper", duration: 60s }
    @assert run.events.length > 0
    @save_run
}
"#;
        let module = parse_quant_script_module(source).unwrap();
        let plan = extract_test_plan(&module).unwrap();
        assert_eq!(plan.scenario_name, "场景一：BTC双均线");
        assert_eq!(plan.cover, vec!["P-03", "STRAT-001"]);
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].name, "编译策略");
        assert_eq!(plan.steps[0].actions.len(), 2);
        assert_eq!(plan.steps[1].name, "Paper运行");
        assert_eq!(plan.steps[1].actions.len(), 3);
    }

    #[test]
    fn splits_strategy_and_test_items() {
        let source = r#"
import math

fn strategy() {
    let closes = fetch("BTCUSDT", interval="1d", lookback=300)?
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}

@test {
    name: "测试"
    cover: ["T-001"]
}

@step("运行") {
    @run { mode: "paper", duration: 30s }
}
"#;
        let module = parse_quant_script_module(source).unwrap();
        let (strategy_module, test_plans) = split_test_items(&module);
        assert_eq!(strategy_module.items.len(), 2); // import + fn
        assert_eq!(test_plans.len(), 1);
        assert_eq!(test_plans[0].scenario_name, "测试");
    }

    #[test]
    fn returns_none_for_module_without_tests() {
        let source = r#"
fn strategy() {
    emit Intent("BUY", instrument="BTCUSDT", quantity=1.0)
}
"#;
        let module = parse_quant_script_module(source).unwrap();
        assert!(extract_test_plan(&module).is_none());
    }
}

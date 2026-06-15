use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum CliCommand {
    Serve,
    PrintHelp,
    StrategyIrValidate { path: PathBuf },
    V4Run { graph_id_or_path: String },
}

pub(super) fn cli_usage() -> &'static str {
    "用法:\n  quantpilot                              启动 QuantPilot API 服务器\n  quantpilot v4-run <graph_id|path>        启动 v4 PaperSimulated 策略\n  quantpilot strategy-ir validate <路径>     加载并验证 Strategy IR JSON 文件\n  quantpilot credential <子命令> [参数]      管理 API 凭证 (set|get|list|delete)\n  quantpilot --help                        显示此帮助"
}

pub(super) fn parse_cli_command_from<I, S>(args: I) -> anyhow::Result<CliCommand>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    let mut iter = args.iter();
    let _program = iter.next();

    match iter.next().map(String::as_str) {
        None => Ok(CliCommand::Serve),
        Some("-h" | "--help" | "help") => Ok(CliCommand::PrintHelp),
        Some("v4-run") => {
            let graph_id_or_path = iter.next().cloned().ok_or_else(|| {
                anyhow::anyhow!(
                    "missing graph id or file path for `v4-run`\n\n{}",
                    cli_usage()
                )
            })?;
            if let Some(extra) = iter.next() {
                bail!(
                    "unexpected extra argument for `v4-run`: {}\n\n{}",
                    extra,
                    cli_usage()
                );
            }
            Ok(CliCommand::V4Run { graph_id_or_path })
        }
        Some("strategy-ir") => match iter.next().map(String::as_str) {
            Some("validate") => {
                let path = iter.next().cloned().map(PathBuf::from).ok_or_else(|| {
                    anyhow::anyhow!(
                        "missing Strategy IR JSON path for `strategy-ir validate`\n\n{}",
                        cli_usage()
                    )
                })?;
                if let Some(extra) = iter.next() {
                    bail!(
                        "unexpected extra argument for `strategy-ir validate`: {}\n\n{}",
                        extra,
                        cli_usage()
                    );
                }
                Ok(CliCommand::StrategyIrValidate { path })
            }
            Some("-h" | "--help" | "help") => Ok(CliCommand::PrintHelp),
            Some(other) => bail!(
                "不支持的 `strategy-ir` 子命令: {}\n\n{}",
                other,
                cli_usage()
            ),
            None => bail!("缺少 `strategy-ir` 子命令\n\n{}", cli_usage()),
        },
        Some(other) => bail!("不支持的命令: {}\n\n{}", other, cli_usage()),
    }
}

#[cfg_attr(test, allow(dead_code))]
pub(super) fn print_cli_usage() {
    println!("{}", cli_usage());
}

pub(super) fn print_strategy_ir_summary(path: &FsPath, strategy_ir: &StrategyIr) {
    println!("Strategy IR file: {}", path.display());
    println!("  strategy_id: {}", strategy_ir.metadata.strategy_id);
    println!("  name: {}", strategy_ir.metadata.name);
    println!("  signals: {}", strategy_ir.signals.len());
    println!(
        "  data_requirements: {}",
        strategy_ir.data_requirements.len()
    );
}

pub(super) fn parse_strategy_ir_json(source: &str) -> anyhow::Result<StrategyIr> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let strategy_ir: StrategyIr =
        serde_json::from_str(source).context("解析 Strategy IR JSON 失败")?;
    strategy_ir
        .validate()
        .context("验证 Strategy IR 负载失败")?;
    Ok(strategy_ir)
}

pub(super) async fn validate_strategy_ir_file(path: PathBuf) -> anyhow::Result<()> {
    let source = fs::read_to_string(&path)
        .await
        .with_context(|| format!("读取 Strategy IR 文件 `{}` 失败", path.display()))?;
    let strategy_ir = parse_strategy_ir_json(&source)
        .with_context(|| format!("无效的 Strategy IR 文件 `{}`", path.display()))?;
    print_strategy_ir_summary(&path, &strategy_ir);
    Ok(())
}

pub(super) async fn run_v4_strategy_from_cli(graph_id_or_path: String) -> anyhow::Result<()> {
    let source_path = resolve_v4_run_source_path(&graph_id_or_path);
    if !source_path.exists() {
        anyhow::bail!("策略图不存在: {}", graph_id_or_path);
    }

    let source = fs::read_to_string(&source_path)
        .await
        .with_context(|| format!("读取 v4 QS 文件 `{}` 失败", source_path.display()))?;
    let source = source.trim_start_matches('\u{feff}');
    let audit = quantscript::audit_v4_quant_script_static(&source, &cli_v4_static_bundle());
    let handoff = quantscript::build_v4_qs_runtime_handoff(&audit);
    if !handoff.accepted_for_runtime_handoff {
        anyhow::bail!(
            "v4 runtime handoff rejected: {}",
            handoff.diagnostics.join("; ")
        );
    }
    let graph = audit
        .parsed_graph
        .ok_or_else(|| anyhow::anyhow!("v4 static audit did not produce a machine graph"))?;
    let initial_event = cli_v4_initial_event(&graph, crate::current_time_ms())?;
    let graph_id = graph.graph_id.clone();
    let mut runtime = qrpc_runtime::V4PaperSimulatedRuntime::new_with_execution_capabilities(
        graph,
        cli_runtime_simulated_v4_matrix("paper-local"),
        vec![qrpc_core_ir::v4::ExecutionCapabilityKind::Market],
    )?;
    let output = runtime.submit_event(initial_event)?;

    println!("v4 runtime graph_id: {}", graph_id);
    println!("v4 runtime events: {}", output.events.len());
    for event in &output.events {
        println!(
            "  #{} {} <- {} @ {}",
            event.sequence, event.event_type, event.source, event.ts_ms
        );
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&output.memory_snapshot)
            .context("序列化 v4 runtime memory snapshot 失败")?
    );
    Ok(())
}

fn resolve_v4_run_source_path(graph_id_or_path: &str) -> PathBuf {
    let path = PathBuf::from(graph_id_or_path);
    if path.exists() || path.extension().is_some() {
        path
    } else {
        PathBuf::from("storage")
            .join("graphs")
            .join(format!("{}.qs", graph_id_or_path))
    }
}

fn cli_v4_initial_event(
    graph: &qrpc_core_ir::v4::V4MachineGraphContract,
    ts_ms: u64,
) -> anyhow::Result<qrpc_runtime::V4RuntimeInputEvent> {
    let spec = graph
        .event_catalog
        .as_ref()
        .and_then(|catalog| {
            catalog
                .events
                .iter()
                .find(|event| {
                    event.source_kind == qrpc_core_ir::v4::MachineEventSourceKind::Runtime
                })
                .or_else(|| catalog.events.first())
        })
        .ok_or_else(|| anyhow::anyhow!("v4 graph requires at least one event catalog entry"))?;
    let mut payload = serde_json::Map::new();
    for field in &spec.payload_fields {
        payload.insert(
            field.name.clone(),
            cli_default_v4_payload_value(field, graph.graph_id.as_str()),
        );
    }
    Ok(qrpc_runtime::V4RuntimeInputEvent {
        event_type: spec.event_type.clone(),
        source: "runtime".to_string(),
        payload: serde_json::Value::Object(payload),
        ts_ms,
    })
}

fn cli_default_v4_payload_value(
    field: &qrpc_core_ir::v4::MachineEventPayloadField,
    graph_id: &str,
) -> serde_json::Value {
    match field.type_name.trim().to_ascii_lowercase().as_str() {
        "string" | "symbol" | "venue" | "account" | "side" | "position_side" | "order_type"
        | "time_in_force" | "freshness" | "runtime_mode" | "order_permission" => {
            if field.name == "strategy_id" {
                serde_json::Value::String(graph_id.to_string())
            } else {
                serde_json::Value::String(field.name.clone())
            }
        }
        "bool" | "boolean" => serde_json::Value::Bool(true),
        "u64" | "uint" => serde_json::Value::Number(serde_json::Number::from(0_u64)),
        "i64" | "int" | "integer" => serde_json::Value::Number(serde_json::Number::from(0_i64)),
        "f64" | "decimal" | "number" | "price" | "quantity" | "notional" | "percent" | "ratio"
        | "fee" | "slippage" | "leverage" => serde_json::Number::from_f64(0.0)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        "object" | "map" => serde_json::json!({}),
        "array" | "list" => serde_json::json!([]),
        _ if field.nullable => serde_json::Value::Null,
        _ => serde_json::Value::String(field.name.clone()),
    }
}

fn cli_v4_static_bundle() -> qrpc_core_ir::v4::V4StaticContractBundle {
    qrpc_core_ir::v4::V4StaticContractBundle {
        venue_matrices: vec![cli_runtime_simulated_v4_matrix("paper-local")],
        ..qrpc_core_ir::v4::V4StaticContractBundle::default()
    }
}

fn cli_runtime_simulated_v4_matrix(
    venue_id: impl Into<String>,
) -> qrpc_core_ir::v4::VenueCapabilityMatrix {
    let mut matrix = qrpc_core_ir::v4::unsupported_v4_first_wave_matrix(venue_id);
    for entry in &mut matrix.capabilities {
        if matches!(
            entry.capability,
            qrpc_core_ir::v4::ExecutionCapabilityKind::Market
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Limit
                | qrpc_core_ir::v4::ExecutionCapabilityKind::StopMarket
                | qrpc_core_ir::v4::ExecutionCapabilityKind::StopLimit
                | qrpc_core_ir::v4::ExecutionCapabilityKind::TakeProfitMarket
                | qrpc_core_ir::v4::ExecutionCapabilityKind::TakeProfitLimit
                | qrpc_core_ir::v4::ExecutionCapabilityKind::OcoBracket
                | qrpc_core_ir::v4::ExecutionCapabilityKind::TrailingStop
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Gtc
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Ioc
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Fok
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Day
                | qrpc_core_ir::v4::ExecutionCapabilityKind::Gtd
                | qrpc_core_ir::v4::ExecutionCapabilityKind::PostOnly
                | qrpc_core_ir::v4::ExecutionCapabilityKind::ReduceOnly
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CloseOnly
                | qrpc_core_ir::v4::ExecutionCapabilityKind::ClientOrderId
                | qrpc_core_ir::v4::ExecutionCapabilityKind::OpenLong
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CloseLong
                | qrpc_core_ir::v4::ExecutionCapabilityKind::OpenShort
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CloseShort
                | qrpc_core_ir::v4::ExecutionCapabilityKind::CancelReplaceAmend
        ) {
            entry.source = qrpc_core_ir::v4::CapabilitySupportSource::RuntimeSimulated;
            entry.supported_modes = vec![qrpc_core_ir::v4::RuntimeTradingMode::PaperSimulated];
        }
    }
    matrix
}

pub fn handle_credential_command(args: &[String]) -> anyhow::Result<()> {
    if args.len() < 2 {
        eprintln!("用法: quantpilot credential <set|get|list|delete> [参数]");
        eprintln!("  set <标签>          交互式输入 (不在 shell history 中留存凭证明文)");
        eprintln!(
            "  set <标签> --stdin   从 stdin 读取 JSON: {{\"key\":\"...\",\"secret\":\"...\"}}"
        );
        eprintln!("  get <标签>           显示标签下的全部字段");
        eprintln!("  list                 列出所有已存储标签");
        eprintln!("  delete <标签>         删除指定标签的全部字段");
        return Ok(());
    }

    let sub = &args[1];
    let vault = crate::credential_vault::CredentialVault::load()?;
    // 将 vault 中的字段值注册到日志脱敏模块
    crate::safe_log::register_credential_patterns(vault.extract_secret_patterns());

    match sub.as_str() {
        "set" => {
            let service = args.get(2).ok_or_else(|| anyhow::anyhow!("缺少标签名"))?;
            let use_stdin = args.iter().any(|a| a == "--stdin");
            let fields: std::collections::BTreeMap<String, String>;

            if use_stdin {
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| anyhow::anyhow!("读取 stdin 失败: {}", e))?;
                fields = serde_json::from_str(&input)
                    .map_err(|e| anyhow::anyhow!("stdin 格式错误, 需要 JSON 对象: {}", e))?;
            } else {
                // 交互式输入: 敏感字段不经过命令行参数或 shell history
                use std::io::{self, Write};
                let mut key = String::new();
                let mut secret = String::new();
                let mut passphrase = String::new();

                eprint!("请输入 api_key: ");
                io::stderr().flush().ok();
                io::stdin()
                    .read_line(&mut key)
                    .map_err(|e| anyhow::anyhow!("读取输入失败: {}", e))?;
                key = key.trim().to_string();

                eprint!("请输入 secret: ");
                io::stderr().flush().ok();
                io::stdin()
                    .read_line(&mut secret)
                    .map_err(|e| anyhow::anyhow!("读取输入失败: {}", e))?;
                secret = secret.trim().to_string();

                eprint!("请输入 passphrase (可选, 直接回车跳过): ");
                io::stderr().flush().ok();
                io::stdin()
                    .read_line(&mut passphrase)
                    .map_err(|e| anyhow::anyhow!("读取输入失败: {}", e))?;
                passphrase = passphrase.trim().to_string();

                if key.is_empty() || secret.is_empty() {
                    anyhow::bail!("api_key 和 secret 不能为空");
                }

                let mut f: std::collections::BTreeMap<String, String> =
                    std::collections::BTreeMap::new();
                f.insert("key".to_string(), key);
                f.insert("secret".to_string(), secret);
                if !passphrase.is_empty() {
                    f.insert("passphrase".to_string(), passphrase);
                }
                fields = f;
            }

            if fields.is_empty() {
                anyhow::bail!("需要至少一个字段");
            }
            vault.set_service(service, fields)?;
            println!("凭证已存储。标签: {}", service);
        }
        "get" => {
            let service = args.get(2).ok_or_else(|| anyhow::anyhow!("缺少标签名"))?;
            let reveal = args.iter().any(|a| a == "--reveal");
            match vault.get_service(service) {
                Some(fields) => {
                    if reveal {
                        for (k, v) in &fields {
                            println!("  {} = {}", k, v.as_str());
                        }
                    } else {
                        let names: Vec<&String> = fields.keys().collect();
                        println!(
                            "标签 '{}' 包含字段: {} (使用 --reveal 查看值)",
                            service,
                            names
                                .iter()
                                .map(|s| s.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                }
                None => println!("标签 '{}' 不存在", service),
            }
        }
        "list" => {
            let services = vault.list_services();
            if services.is_empty() {
                println!("(无已存储凭证)");
            } else {
                for s in &services {
                    println!("  {}", s);
                }
            }
        }
        "delete" => {
            let service = args.get(2).ok_or_else(|| anyhow::anyhow!("缺少标签名"))?;
            vault.delete_service(service)?;
            println!("凭证已删除。标签: {}", service);
        }
        _ => {
            anyhow::bail!("未知子命令: {}。可用: set, get, list, delete", sub);
        }
    }
    Ok(())
}

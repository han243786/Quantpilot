use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum CliCommand {
    Serve,
    PrintHelp,
    StrategyIrValidate { path: PathBuf },
}

pub(super) fn cli_usage() -> &'static str {
    "Usage:\n  quantpilot                      Start the QuantPilot API server\n  quantpilot strategy-ir validate <path>\n                                  Load and validate a Strategy IR JSON file\n  quantpilot --help               Show this help"
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

pub fn handle_credential_command(args: &[String]) -> anyhow::Result<()> {
    if args.len() < 2 {
        eprintln!("用法: quantpilot credential <set|get|list|delete> [参数]");
        eprintln!("  set <标签> --key=<KEY> --secret=<SECRET> [--passphrase=<PASSPHRASE>] [--<自定义字段>=<值> ...]");
        eprintln!("  get <标签>");
        eprintln!("  list");
        eprintln!("  delete <标签>");
        return Ok(());
    }

    let sub = &args[1];
    let mut vault = crate::credential_vault::CredentialVault::load()?;

    match sub.as_str() {
        "set" => {
            let service = args.get(2).ok_or_else(|| anyhow::anyhow!("缺少标签名"))?;
            let mut fields: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
            for arg in &args[3..] {
                if let Some(rest) = arg.strip_prefix("--") {
                    if let Some((k, v)) = rest.split_once('=') {
                        fields.insert(k.to_string(), v.to_string());
                    }
                }
            }
            if fields.is_empty() {
                anyhow::bail!("需要至少一个字段, 例如 --key=xxx --secret=yyy");
            }
            vault.set_service(service, fields)?;
            println!("凭证已存储。标签: {}", service);
        }
        "get" => {
            let service = args.get(2).ok_or_else(|| anyhow::anyhow!("缺少标签名"))?;
            match vault.get_service(service) {
                Some(fields) => {
                    for (k, v) in &fields {
                        println!("  {} = {}", k, v);
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

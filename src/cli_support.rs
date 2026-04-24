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
                "unsupported `strategy-ir` subcommand: {}\n\n{}",
                other,
                cli_usage()
            ),
            None => bail!("missing `strategy-ir` subcommand\n\n{}", cli_usage()),
        },
        Some(other) => bail!("unsupported command: {}\n\n{}", other, cli_usage()),
    }
}

#[cfg_attr(test, allow(dead_code))]
pub(super) fn print_cli_usage() {
    println!("{}", cli_usage());
}

pub(super) fn print_strategy_ir_summary(path: &PathBuf, strategy_ir: &StrategyIr) {
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
        serde_json::from_str(source).context("failed to parse Strategy IR JSON")?;
    strategy_ir
        .validate()
        .context("failed to validate Strategy IR payload")?;
    Ok(strategy_ir)
}

pub(super) async fn validate_strategy_ir_file(path: PathBuf) -> anyhow::Result<()> {
    let source = fs::read_to_string(&path)
        .await
        .with_context(|| format!("failed to read Strategy IR file `{}`", path.display()))?;
    let strategy_ir = parse_strategy_ir_json(&source)
        .with_context(|| format!("invalid Strategy IR file `{}`", path.display()))?;
    print_strategy_ir_summary(&path, &strategy_ir);
    Ok(())
}

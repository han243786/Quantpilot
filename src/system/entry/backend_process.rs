use crate::cli_support::{
    self, parse_cli_command_from, print_cli_usage, run_v4_strategy_from_cli,
    validate_strategy_ir_file, CliCommand,
};
use std::env;

pub async fn run_server() -> anyhow::Result<()> {
    initialize_process_environment();
    dispatch_process_command().await
}

fn initialize_process_environment() {
    let _ = dotenvy::dotenv();

    let log_format = env::var("QUANTPILOT_LOG_FORMAT").unwrap_or_else(|_| "compact".to_string());
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr);
    if log_format == "json" {
        subscriber.json().init();
    } else {
        subscriber.compact().init();
    }

    if env::var("QUANTPILOT_DEV").unwrap_or_default() == "true" {
        safe_eprintln!("[启动] DEV 模式已启用 — 瞬态数据 TTL 缩短，强制启动清理");
    }

    std::panic::set_hook(Box::new(|info| {
        let msg = format!("{}", info);
        eprintln!(
            "[panic] {} — 服务将退出",
            crate::safe_log::sanitize_secrets(&msg)
        );
    }));
}

async fn dispatch_process_command() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "credential" {
        if let Err(error) = cli_support::handle_credential_command(&args[1..]) {
            safe_eprintln!("错误: {}", error);
            std::process::exit(1);
        }
        return Ok(());
    }

    match parse_cli_command_from(env::args())? {
        CliCommand::Serve => crate::run_api_server().await,
        CliCommand::PrintHelp => {
            print_cli_usage();
            Ok(())
        }
        CliCommand::StrategyIrValidate { path } => validate_strategy_ir_file(path).await,
        CliCommand::V4Run { graph_id_or_path } => run_v4_strategy_from_cli(graph_id_or_path).await,
    }
}

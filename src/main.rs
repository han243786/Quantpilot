// QuantPilot v3.5.0 — 二进制入口
// 核心逻辑位于 lib.rs, 本文件仅负责启动 tokio 运行时并调用 run_server.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    quantpilot::run_server().await
}

use qrpc_core::PluginManifest;
use serde::{Deserialize, Serialize};

/// v1.0.0 插件市场元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketMetadata {
    pub repo_url: String,
    pub index_version: String,
    pub index_hash: String,
    pub plugins: Vec<PluginSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginSummary {
    pub plugin_id: String,
    pub version: String,
    pub display_name: String,
    pub summary: String,
    pub plugin_type: String,
    pub manifest_hash: String,
    pub download_url: String,
}

/// 插件市场客户端 — 远端拉取 index.json + 本地协议校验
pub struct PluginMarketClient {
    repo_url: String,
    client: reqwest::Client,
}

const MARKET_TIMEOUT_SECS: u64 = 30;
const MARKET_MAX_BODY_BYTES: u64 = 2 * 1024 * 1024; // 2 MB

impl PluginMarketClient {
    pub fn new(repo_url: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(MARKET_TIMEOUT_SECS))
            .build()
            .expect("PluginMarketClient reqwest 客户端构建失败");
        Self {
            repo_url: repo_url.into(),
            client,
        }
    }

    /// 从远端拉取插件市场索引
    pub async fn fetch_index(&self) -> Result<MarketMetadata, String> {
        let index_url = format!("{}/index.json", self.repo_url.trim_end_matches('/'));
        let response = self.client.get(&index_url).send().await
            .map_err(|e| format!("无法连接插件市场: {e}"))?;
        let body = response
            .text()
            .await
            .map_err(|e| format!("读取市场响应失败: {e}"))?;
        if body.len() as u64 > MARKET_MAX_BODY_BYTES {
            return Err("市场索引超过大小限制 (2 MB)".to_string());
        }
        let metadata: MarketMetadata =
            serde_json::from_str(&body).map_err(|e| format!("市场索引 JSON 解析失败: {e}"))?;
        Ok(metadata)
    }

    /// 拉取单个插件的 manifest
    pub async fn fetch_manifest(&self, summary: &PluginSummary) -> Result<PluginManifest, String> {
        let response = self.client.get(&summary.download_url).send().await
            .map_err(|e| format!("无法下载插件 {}: {e}", summary.plugin_id))?;
        let body = response
            .text()
            .await
            .map_err(|e| format!("读取插件 manifest 失败: {e}"))?;
        if body.len() as u64 > MARKET_MAX_BODY_BYTES {
            return Err(format!("插件 {} manifest 超过大小限制 (2 MB)", summary.plugin_id));
        }
        let manifest: PluginManifest = serde_json::from_str(&body)
            .map_err(|e| format!("插件 {} manifest JSON 解析失败: {e}", summary.plugin_id))?;

        manifest.validate().map_err(|errs| {
            format!(
                "插件 {} manifest 协议校验失败: {:?}",
                summary.plugin_id, errs
            )
        })?;

        Ok(manifest)
    }
}

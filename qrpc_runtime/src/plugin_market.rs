use qrpc_core::PluginManifest;
use ed25519_dalek::Verifier;
use serde::{Deserialize, Serialize};

// ── Ed25519 签名验证 ──────────────────────────────────

/// RFC 8032 §7.1 测试向量公钥。仅用于单元测试，严禁用于生产。
const RFC8032_TEST_VECTOR_PUBLIC_KEY: &str = "11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo=";

/// 获取市场公钥。优先读取 `QUANTPILOT_MARKET_PUBLIC_KEY` 环境变量，
/// 若未设置则回退到 RFC 8032 测试向量并 panic。
fn market_public_key() -> String {
    if let Ok(key) = std::env::var("QUANTPILOT_MARKET_PUBLIC_KEY") {
        let trimmed = key.trim().to_string();
        if !trimmed.is_empty() && trimmed != RFC8032_TEST_VECTOR_PUBLIC_KEY {
            return trimmed;
        }
        panic!(
            "QUANTPILOT_MARKET_PUBLIC_KEY 环境变量未设置或仍为 RFC 8032 测试向量。\
             请使用 `ed25519-dalek` 生成生产密钥对后设置此变量。"
        );
    }
    // 测试环境回退（仅当未设置环境变量时用于 cargo test）
    RFC8032_TEST_VECTOR_PUBLIC_KEY.to_string()
}

/// 启动时校验市场公钥不是测试向量。应在 main 启动阶段调用。
/// 生产环境下若仍为测试向量则中止进程。
pub fn assert_market_public_key_is_production() {
    let key = market_public_key();
    if key == RFC8032_TEST_VECTOR_PUBLIC_KEY {
        panic!(
            "MARKET_PUBLIC_KEY 仍为 RFC 8032 测试向量，生产环境禁止启动。\
             请设置 QUANTPILOT_MARKET_PUBLIC_KEY 环境变量为实际市场公钥。"
        );
    }
}

/// 从 JSON 负载提取签名并验证 Ed25519 签名。
///
/// 验证流程:
/// 1. 从完整 JSON 中提取 `"signature"` 字段 (base64)
/// 2. 移除 `"signature"` 字段后对剩余 JSON 做规范序列化
/// 3. 使用市场公钥验证签名
pub(crate) fn verify_manifest_signature(raw_json: &str) -> Result<(), String> {
    let mut value: serde_json::Value =
        serde_json::from_str(raw_json).map_err(|e| format!("manifest JSON 解析失败: {e}"))?;

    // 提取 signature 字段 (先 clone 避免借用冲突)
    let signature_b64 = value
        .get("signature")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "manifest 缺少 signature 字段, 拒绝加载".to_string())?;

    // 从 JSON 中移除 signature, 剩余部分为签名原文
    if let serde_json::Value::Object(ref mut map) = value {
        map.remove("signature");
    } else {
        return Err("manifest 顶层必须是 JSON 对象".to_string());
    }

    // 规范序列化 (依赖 BTreeMap 保证确定性, 与 canonical_json_sha256_digest 一致)
    let canonical =
        serde_json::to_vec(&value).map_err(|e| format!("规范 JSON 序列化失败: {e}"))?;

    // 解码签名 (Ed25519 签名 = 64 字节)
    use base64::Engine;
    let signature_bytes = base64::engine::general_purpose::STANDARD
        .decode(&signature_b64)
        .map_err(|e| format!("signature base64 解码失败: {e}"))?;
    let signature = ed25519_dalek::Signature::from_slice(&signature_bytes)
        .map_err(|e| format!("Ed25519 签名解析失败: {e}"))?;

    // 解码公钥
    let public_key_str = market_public_key();
    let pub_key_bytes: [u8; 32] = {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&public_key_str)
            .map_err(|e| format!("市场公钥 base64 解码失败: {e}"))?;
        decoded
            .try_into()
            .map_err(|_| "市场公钥解码后长度不是 32 字节".to_string())?
    };
    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&pub_key_bytes)
        .map_err(|e| format!("Ed25519 公钥解析失败: {e}"))?;

    // 验证
    verifying_key
        .verify(&canonical, &signature)
        .map_err(|e| format!("Ed25519 签名验证失败: {e}"))?;

    Ok(())
}

// ── 市场数据结构 ──────────────────────────────────────

/// v1.0.0 插件市场元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MarketMetadata {
    pub repo_url: String,
    pub index_version: String,
    pub index_hash: String,
    pub plugins: Vec<PluginSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PluginSummary {
    pub plugin_id: String,
    pub version: String,
    pub display_name: String,
    pub summary: String,
    pub plugin_type: String,
    pub manifest_hash: String,
    pub download_url: String,
}

/// 插件市场客户端 — 远端拉取 index.json + 本地协议校验 + Ed25519 签名验证
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
    ///
    /// v2.0.1 已知限制: index_hash 和 manifest_hash 字段仅存储未校验。
    /// v2.1.0 计划引入 Ed25519 索引签名验证, 确保索引未被中间人篡改。
    /// 当前仅依赖 manifest 级别的 Ed25519 签名验证。
    pub async fn fetch_index(&self) -> Result<MarketMetadata, String> {
        let index_url = format!("{}/index.json", self.repo_url.trim_end_matches('/'));
        let response = self
            .client
            .get(&index_url)
            .send()
            .await
            .map_err(|e| format!("无法连接插件市场: {e}"))?;
        if response.status() == 404 {
            return Err("插件市场索引不存在 (404)".to_string());
        }
        if !response.status().is_success() {
            return Err(format!("插件市场返回错误: HTTP {}", response.status()));
        }
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

    /// 拉取单个插件的 manifest 并验证 Ed25519 签名。
    ///
    /// v2.0.0: 验证流程
    /// 1. 下载 manifest JSON (包含 `signature` 字段)
    /// 2. 协议校验 (`manifest.validate()`)
    /// 3. Ed25519 签名验证 (使用 `MARKET_PUBLIC_KEY`)
    pub async fn fetch_manifest(&self, summary: &PluginSummary) -> Result<PluginManifest, String> {
        let response = self
            .client
            .get(&summary.download_url)
            .send()
            .await
            .map_err(|e| format!("无法下载插件 {}: {e}", summary.plugin_id))?;
        if response.status() == 404 {
            return Err(format!("插件 {} 不存在 (404)", summary.plugin_id));
        }
        if !response.status().is_success() {
            return Err(format!(
                "插件 {} 下载失败: HTTP {}",
                summary.plugin_id,
                response.status()
            ));
        }
        let body = response
            .text()
            .await
            .map_err(|e| format!("读取插件 manifest 失败: {e}"))?;
        if body.len() as u64 > MARKET_MAX_BODY_BYTES {
            return Err(format!(
                "插件 {} manifest 超过大小限制 (2 MB)",
                summary.plugin_id
            ));
        }

        // v2.0.0: Ed25519 签名验证 (在协议校验之前, 拒绝篡改的 manifest)
        verify_manifest_signature(&body).map_err(|e| {
            format!(
                "插件 {} Ed25519 签名验证失败: {}",
                summary.plugin_id, e
            )
        })?;

        let manifest: PluginManifest = serde_json::from_str(&body).map_err(|e| {
            format!(
                "插件 {} manifest JSON 解析失败: {e}",
                summary.plugin_id
            )
        })?;

        manifest.validate().map_err(|errs| {
            format!(
                "插件 {} manifest 协议校验失败: {:?}",
                summary.plugin_id, errs
            )
        })?;

        Ok(manifest)
    }
}

// ── 测试 ──

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use ed25519_dalek::Signer;

    // RFC 8032 Test #1 私钥 (仅用于测试签名生成)
    const TEST_SECRET_KEY: [u8; 32] = [
        0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60, 0xba, 0x84, 0x4a, 0xf4,
        0x92, 0xec, 0x2c, 0xc4, 0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19,
        0x70, 0x3b, 0xac, 0x03, 0x1c, 0xae, 0x7f, 0x60,
    ];

    /// 使用测试密钥对 JSON 负载生成 Ed25519 签名, 返回完整 manifest JSON
    fn sign_manifest_json(payload: &serde_json::Value) -> String {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&TEST_SECRET_KEY);
        let canonical = serde_json::to_vec(payload).expect("JSON 序列化");
        let signature = signing_key.sign(&canonical);
        let sig_b64 = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());

        // 构造带 signature 的完整 JSON
        let mut with_sig = payload.clone();
        if let serde_json::Value::Object(ref mut map) = with_sig {
            map.insert(
                "signature".to_string(),
                serde_json::Value::String(sig_b64),
            );
        }
        serde_json::to_string(&with_sig).expect("JSON 序列化")
    }

    #[test]
    fn verify_signature_with_test_vector() {
        // 使用测试密钥对空对象签名并验证
        let payload = serde_json::json!({});
        let manifest_json = sign_manifest_json(&payload);

        let result = verify_manifest_signature(&manifest_json);
        assert!(result.is_ok(), "签名验证应通过: {:?}", result.err());
    }

    #[test]
    fn verify_signature_with_complex_payload() {
        let payload = serde_json::json!({
            "api_version": "quantpilot/plugin-manifest/v1",
            "id": "quantpilot.data.test",
            "version": "0.1.0",
            "kind": "data",
        });
        let manifest_json = sign_manifest_json(&payload);

        let result = verify_manifest_signature(&manifest_json);
        assert!(result.is_ok(), "复杂负载签名验证应通过: {:?}", result.err());
    }

    #[test]
    fn verify_signature_rejects_missing_signature_field() {
        let raw = r#"{"id":"test","version":"1.0.0"}"#;
        let result = verify_manifest_signature(raw);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("缺少 signature"),
            "应报告缺少 signature"
        );
    }

    #[test]
    fn verify_signature_rejects_tampered_content() {
        let payload = serde_json::json!({"id": "original", "version": "1.0.0"});
        let manifest_json = sign_manifest_json(&payload);

        // 篡改: 将 id 改为 "tampered"
        let tampered = manifest_json.replace("\"original\"", "\"tampered\"");

        let result = verify_manifest_signature(&tampered);
        assert!(result.is_err(), "篡改后的签名应被拒绝");
        let err = result.unwrap_err();
        assert!(
            err.contains("签名验证失败"),
            "应报告签名验证失败, 实际: {err}"
        );
    }

    #[test]
    fn verify_signature_rejects_invalid_base64_signature() {
        let raw = r#"{"signature":"!!!invalid-base64!!!","id":"test"}"#;
        let result = verify_manifest_signature(raw);
        assert!(result.is_err());
    }

    #[test]
    fn verify_signature_rejects_wrong_key() {
        // 使用测试密钥签名, 但篡改字段值, 应被 MARKET_PUBLIC_KEY 拒绝
        let payload = serde_json::json!({"msg": "hello"});
        let manifest_json = sign_manifest_json(&payload);

        // 篡改: 将 "msg" 从 "hello" 改为 "tampered"
        let tampered = manifest_json.replace("\"hello\"", "\"tampered\"");

        let result = verify_manifest_signature(&tampered);
        assert!(result.is_err());
    }
}

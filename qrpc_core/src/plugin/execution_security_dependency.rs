use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginExecution {
    pub engine: PluginExecutionEngine,
    pub entrypoint: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginExecutionEngine {
    Builtin,
    QuantScript,
    Native,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginCompatibility {
    pub core_ir_version: String,
    pub capability_api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginSecurity {
    pub max_compute_ms: u64,
    pub max_memory_mb: u64,
    #[serde(default)]
    pub allow_network: bool,
    // v2.0.0: 纭檺鍒?鈥?瀛愯繘绋嬫矙绠卞疄闄呮墽琛? 瑕嗙洊 max_compute_ms
    #[serde(default)]
    pub enforce_max_compute_ms: Option<u64>,
    // v2.0.0: 纭檺鍒?鈥?瀛愯繘绋嬫矙绠卞疄闄呮墽琛? 瑕嗙洊 max_memory_mb
    #[serde(default)]
    pub enforce_max_memory_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginDependency {
    pub plugin_id: String,
    pub version_req: String,
}

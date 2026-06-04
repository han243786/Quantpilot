use std::collections::BTreeMap;

use super::{ExtensionPoint, PluginManifest};

#[derive(Debug, Default, Clone)]
pub struct PluginRegistry {
    manifests: BTreeMap<String, PluginManifest>,
}

impl PluginRegistry {
    pub fn register(&mut self, manifest: PluginManifest) -> Result<(), Vec<String>> {
        manifest.validate()?;
        if self.manifests.contains_key(&manifest.id) {
            return Err(vec![format!("插件 `{}` 已注册", manifest.id)]);
        }
        self.manifests.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    pub fn get(&self, plugin_id: &str) -> Option<&PluginManifest> {
        self.manifests.get(plugin_id)
    }

    // v2.1.0: 卸载插件时移除注册信息
    pub fn remove(&mut self, plugin_id: &str) -> Option<PluginManifest> {
        self.manifests.remove(plugin_id)
    }

    pub fn manifests_for_extension_point(
        &self,
        extension_point: ExtensionPoint,
    ) -> Vec<&PluginManifest> {
        self.manifests
            .values()
            .filter(|manifest| manifest.extension_points.contains(&extension_point))
            .collect()
    }

    pub fn manifests(&self) -> Vec<&PluginManifest> {
        self.manifests.values().collect()
    }
}

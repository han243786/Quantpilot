use super::{ConfigGenerationEntry, RuntimeCoordinator};
use anyhow::Result;
use std::sync::atomic::Ordering;

impl RuntimeCoordinator {
    pub fn swap_module_config(
        &mut self,
        module_key: &str,
        config: serde_json::Value,
    ) -> Result<String> {
        self.pending_module_configs
            .insert(module_key.to_string(), config);
        // Use a sequence instead of wall-clock time to keep backtests deterministic.
        let revision_input = serde_json::json!({
            "module_key": module_key,
            "revision_seq": self.config.applied_deployment_revisions.len(),
        });
        let digest = qrpc_core::canonical_json_sha256_digest(&revision_input)?;
        let revision = format!("rev-hotswap-{}", &digest.value[..16]);
        self.config
            .applied_deployment_revisions
            .push(revision.clone());
        const MAX_REVISIONS: usize = 1000;
        if self.config.applied_deployment_revisions.len() > MAX_REVISIONS {
            let excess = self.config.applied_deployment_revisions.len() - MAX_REVISIONS;
            self.config.applied_deployment_revisions.drain(0..excess);
        }
        Ok(revision)
    }

    pub fn apply_pending_module_configs(&mut self) -> Vec<String> {
        let count = self.pending_module_configs.len();
        self.pending_module_configs.clear();
        if count > 0 {
            let gen = self.config.config_generation.fetch_add(1, Ordering::SeqCst);
            let now_ms = gen;
            let rev = self
                .config
                .applied_deployment_revisions
                .last()
                .cloned()
                .unwrap_or_else(|| "rev-unknown".to_string());
            if let Ok(mut history) = self.config.config_generation_history.lock() {
                const MAX_CONFIG_HISTORY: usize = 1000;
                let len = history.len();
                if len >= MAX_CONFIG_HISTORY {
                    history.drain(0..len - MAX_CONFIG_HISTORY + 1);
                }
                history.push(ConfigGenerationEntry {
                    generation: gen,
                    activated_at_ms: now_ms,
                    deployment_revision: rev.clone(),
                    parameter_version: format!("gen-{}", gen),
                });
            }
            vec![rev]
        } else {
            Vec::new()
        }
    }

    pub fn current_generation(&self) -> u64 {
        self.config.config_generation.load(Ordering::Relaxed)
    }

    pub fn generation_history(&self) -> Vec<ConfigGenerationEntry> {
        self.config
            .config_generation_history
            .lock()
            .map(|h| h.clone())
            .unwrap_or_default()
    }
}

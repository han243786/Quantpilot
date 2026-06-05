use serde::{Deserialize, Serialize};

use super::{default_true, V4_LEARNING_PIPELINE_CONTRACT_VERSION};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeveloperLearningPipelineContract {
    #[serde(default = "default_learning_pipeline_contract_version")]
    pub schema_version: String,
    #[serde(default = "default_true")]
    pub core_pipeline_in_repo: bool,
    #[serde(default = "default_learning_dir")]
    pub local_learning_dir: String,
    #[serde(default = "default_true")]
    pub local_learning_dir_gitignored: bool,
    #[serde(default = "default_true")]
    pub write_requires_explicit_user_command: bool,
    #[serde(default)]
    pub included_in_regular_gates: bool,
    #[serde(default = "default_true")]
    pub major_closeout_question_required: bool,
    #[serde(default = "default_true")]
    pub owner_first_iteration_only: bool,
}

impl Default for DeveloperLearningPipelineContract {
    fn default() -> Self {
        Self {
            schema_version: V4_LEARNING_PIPELINE_CONTRACT_VERSION.to_string(),
            core_pipeline_in_repo: true,
            local_learning_dir: default_learning_dir(),
            local_learning_dir_gitignored: true,
            write_requires_explicit_user_command: true,
            included_in_regular_gates: false,
            major_closeout_question_required: true,
            owner_first_iteration_only: true,
        }
    }
}

impl DeveloperLearningPipelineContract {
    pub fn validate_static_contract(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_LEARNING_PIPELINE_CONTRACT_VERSION {
            errors.push(format!(
                "schema_version must be `{}`",
                V4_LEARNING_PIPELINE_CONTRACT_VERSION
            ));
        }
        if !self.core_pipeline_in_repo {
            errors.push("core learning pipeline must live in the repository".to_string());
        }
        if self.local_learning_dir != default_learning_dir() {
            errors.push(format!(
                "local_learning_dir must be `{}`",
                default_learning_dir()
            ));
        }
        if !self.local_learning_dir_gitignored {
            errors.push("local learning records must stay gitignored".to_string());
        }
        if !self.write_requires_explicit_user_command {
            errors.push("learning records must require explicit user command".to_string());
        }
        if self.included_in_regular_gates {
            errors.push("learning pipeline must not enter regular mandatory gates".to_string());
        }
        if !self.major_closeout_question_required {
            errors.push("MAJOR closeout must ask the learning pipeline question".to_string());
        }
        if !self.owner_first_iteration_only {
            errors.push("first learning pipeline iteration must stay owner-first".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn default_learning_pipeline_contract_version() -> String {
    V4_LEARNING_PIPELINE_CONTRACT_VERSION.to_string()
}

fn default_learning_dir() -> String {
    "markdown/learning/".to_string()
}

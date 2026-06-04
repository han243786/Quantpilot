use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactDigestAlgorithm {
    Sha256CanonicalJson,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactDigest {
    pub algorithm: ArtifactDigestAlgorithm,
    pub value: String,
}

/// Computes the canonical JSON SHA-256 digest used by artifact snapshots.
pub fn canonical_json_sha256_digest<T: Serialize + ?Sized>(
    value: &T,
) -> serde_json::Result<ArtifactDigest> {
    let canonical = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(&canonical);
    Ok(ArtifactDigest {
        algorithm: ArtifactDigestAlgorithm::Sha256CanonicalJson,
        value: format!("{:x}", hasher.finalize()),
    })
}

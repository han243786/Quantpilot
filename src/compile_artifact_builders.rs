use super::*;

fn artifact_id(prefix: &str, digest: &ArtifactDigest) -> String {
    let short = &digest.value[..digest.value.len().min(12)];
    format!("{prefix}_{short}")
}

fn build_strategy_artifact(
    graph_id: &str,
    compile_id: &str,
    name: &str,
    runtime_mode: &str,
    source_kind: StrategyArtifactSourceKind,
    source_ref: &str,
    extra_metadata: BTreeMap<String, Value>,
) -> anyhow::Result<StrategyArtifact> {
    let mut metadata = BTreeMap::from([(
        "runtime_mode".to_string(),
        Value::String(runtime_mode.to_string()),
    )]);
    metadata.extend(extra_metadata);
    let digest = canonical_json_sha256_digest(&json!({
        "graph_id": graph_id,
        "compile_id": compile_id,
        "strategy_id": graph_id,
        "name": name,
        "source_kind": source_kind,
        "source_ref": source_ref,
        "metadata": metadata,
    }))
    .context("计算策略制品哈希失败")?;
    Ok(StrategyArtifact {
        schema_version: STRATEGY_ARTIFACT_V1_VERSION.to_string(),
        artifact_id: artifact_id("strategy_artifact", &digest),
        graph_id: graph_id.to_string(),
        compile_id: compile_id.to_string(),
        strategy_id: graph_id.to_string(),
        name: name.to_string(),
        source_kind,
        source_ref: source_ref.to_string(),
        metadata,
        digest,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_compile_artifact_bundle(
    graph_id: &str,
    compile_id: &str,
    name: &str,
    runtime_mode: &str,
    source_kind: StrategyArtifactSourceKind,
    source_ref: &str,
    extra_metadata: BTreeMap<String, Value>,
    compiled: &qrpc_core::CompiledRuntimeProtocol,
) -> anyhow::Result<CompileArtifactBundle> {
    let strategy = build_strategy_artifact(
        graph_id,
        compile_id,
        name,
        runtime_mode,
        source_kind,
        source_ref,
        extra_metadata,
    )?;
    let core_ir_digest = canonical_json_sha256_digest(&compiled.core_ir)
        .context("计算 Core IR 制品哈希失败")?;
    let core_ir = CoreIrArtifact {
        schema_version: CORE_IR_ARTIFACT_V1_VERSION.to_string(),
        artifact_id: artifact_id("core_ir_artifact", &core_ir_digest),
        graph_id: graph_id.to_string(),
        compile_id: compile_id.to_string(),
        ir_version: compiled.core_ir.ir_version.clone(),
        digest: core_ir_digest,
        core_ir: compiled.core_ir.clone(),
    };
    let compile_digest = canonical_json_sha256_digest(&json!({
        "graph_id": graph_id,
        "compile_id": compile_id,
        "protocol_name": compiled.protocol_name.clone(),
        "config_hash": compiled.config_hash.clone(),
        "strategy_artifact_id": strategy.artifact_id.clone(),
        "core_ir_artifact_id": core_ir.artifact_id.clone(),
        "runtime_config": compiled.config.clone(),
    }))
    .context("计算编译制品哈希失败")?;
    let compile = CompileArtifact {
        schema_version: COMPILE_ARTIFACT_V1_VERSION.to_string(),
        artifact_id: artifact_id("compile_artifact", &compile_digest),
        graph_id: graph_id.to_string(),
        compile_id: compile_id.to_string(),
        protocol_name: compiled.protocol_name.clone(),
        config_hash: compiled.config_hash.clone(),
        strategy_artifact_id: strategy.artifact_id.clone(),
        core_ir_artifact_id: core_ir.artifact_id.clone(),
        digest: compile_digest,
        runtime_config: compiled.config.clone(),
    };
    Ok(CompileArtifactBundle {
        strategy,
        compile,
        core_ir,
    })
}

use quantscript::{parse_quant_script_module, ScriptModule};

pub(super) fn parse_generated_qs_lines(mut qs_lines: Vec<String>) -> anyhow::Result<ScriptModule> {
    qs_lines.push("}".to_string());
    let qs_source = qs_lines.join("\n");

    parse_quant_script_module(&qs_source)
}

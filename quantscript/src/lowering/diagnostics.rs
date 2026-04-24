pub(crate) fn format_diagnostics(diagnostics: &[crate::Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| match &diagnostic.span {
            Some(span) => format!(
                "{}: {} [{}]",
                diagnostic.code, diagnostic.message, span.label
            ),
            None => format!("{}: {}", diagnostic.code, diagnostic.message),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

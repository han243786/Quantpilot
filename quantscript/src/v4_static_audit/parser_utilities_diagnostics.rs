use crate::{Diagnostic, Span};

pub(super) struct PreparedLine {
    pub(super) number: usize,
    pub(super) text: String,
}

pub(super) fn prepare_lines(input: &str) -> Vec<PreparedLine> {
    input
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let without_slash_comment = line.split_once("//").map(|(head, _)| head).unwrap_or(line);
            let without_comment = without_slash_comment
                .split_once('#')
                .map(|(head, _)| head)
                .unwrap_or(without_slash_comment);
            let text = without_comment.trim();
            if text.is_empty() {
                None
            } else {
                Some(PreparedLine {
                    number: index + 1,
                    text: text.to_string(),
                })
            }
        })
        .collect()
}

pub(super) fn split_words(input: &str) -> Vec<&str> {
    input.split_whitespace().collect()
}

pub(super) fn split_csv_words(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub(super) fn diag(
    code: &'static str,
    message: impl Into<String>,
    line_number: usize,
) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        Some(Span::module(format!("line {line_number}"))),
    )
}

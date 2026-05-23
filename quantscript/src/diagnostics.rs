#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanContext {
    Module,
    Function(String),
    Binding(String),
    Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub context: SpanContext,
    pub label: String,
}

impl Span {
    pub fn module(label: impl Into<String>) -> Self {
        Self {
            context: SpanContext::Module,
            label: label.into(),
        }
    }

    pub fn function(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            context: SpanContext::Function(name.clone()),
            label: name,
        }
    }

    pub fn binding(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            context: SpanContext::Binding(name.clone()),
            label: name,
        }
    }

    pub fn expr(label: impl Into<String>) -> Self {
        Self {
            context: SpanContext::Expr,
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: &'static str,
    pub message: String,
    pub span: Option<Span>,
    /// v1.2.4: 可操作的修复提示
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn error(code: &'static str, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code,
            message: message.into(),
            span,
            hint: None,
        }
    }

    pub fn warning(code: &'static str, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            code,
            message: message.into(),
            span,
            hint: None,
        }
    }

    /// v1.2.4: 带修复提示的错误
    pub fn error_with_hint(
        code: &'static str,
        message: impl Into<String>,
        span: Option<Span>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code,
            message: message.into(),
            span,
            hint: Some(hint.into()),
        }
    }
}

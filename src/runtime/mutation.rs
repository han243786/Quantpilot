// ── Block 5: 运营报表 ──

#[derive(Debug, Deserialize)]
pub(crate) struct OpsDailyQuery {
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AuditWeeklyQuery {
    week_start: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ResearchMonthlyQuery {
    month: Option<String>,
}

import "./backtest-analysis.css";

function SummaryCard({ label, value }) {
  return (
    <div className="analysis-summary-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

export function StrategyRouteBar({ items = [] }) {
  const visibleItems = items.filter(Boolean);
  if (visibleItems.length === 0) return null;

  return (
    <nav className="strategy-route-bar" aria-label="Strategy navigation">
      {visibleItems.map((item, index) => {
        const isCurrent = Boolean(item.current) || index === visibleItems.length - 1;
        return (
          <span key={`${item.label}-${index}`} className="strategy-route-bar__segment">
            {item.onClick && !isCurrent ? (
              <button
                type="button"
                className="strategy-route-bar__link"
                onClick={item.onClick}
              >
                {item.label}
              </button>
            ) : (
              <span
                className={`strategy-route-bar__current${
                  isCurrent ? " strategy-route-bar__current--active" : ""
                }`}
              >
                {item.label}
              </span>
            )}
            {index < visibleItems.length - 1 ? (
              <span className="strategy-route-bar__separator">/</span>
            ) : null}
          </span>
        );
      })}
    </nav>
  );
}

export function AnalysisHero({
  routeItems = [],
  kicker,
  title,
  subtitle,
  meta,
  actions = null,
  summaryItems = [],
  testId = null
}) {
  return (
    <header className="detail-header analysis-hero" data-testid={testId || undefined}>
      <div className="detail-header-main analysis-hero-main">
        <div className="analysis-hero-copy">
          <StrategyRouteBar items={routeItems} />
          <div className="panel-subtitle">{kicker}</div>
          <div className="panel-title">{title}</div>
          <div className="muted-line">{subtitle}</div>
          {meta ? <div className="analysis-hero-meta">{meta}</div> : null}
        </div>
        {actions ? <div className="toolbar-group">{actions}</div> : null}
      </div>
      {summaryItems.length > 0 ? (
        <div className="analysis-summary-grid">
          {summaryItems.map((item) => (
            <SummaryCard key={item.label} label={item.label} value={item.value} />
          ))}
        </div>
      ) : null}
    </header>
  );
}

export function AnalysisSection({
  kicker,
  title,
  summary,
  actions = null,
  className = "",
  testId = null,
  children
}) {
  const classes = ["analysis-section", className].filter(Boolean).join(" ");
  return (
    <section className={classes} data-testid={testId || undefined}>
      <div className="analysis-section-header">
        <div>
          {kicker ? <div className="analysis-section-kicker">{kicker}</div> : null}
          <h2 className="analysis-section-title">{title}</h2>
          {summary ? <p className="analysis-section-summary">{summary}</p> : null}
        </div>
        {actions}
      </div>
      <div className="analysis-section-body">{children}</div>
    </section>
  );
}

export function AnalysisStatusBanner({ variant = "info", children, testId = null }) {
  return (
    <div
      className={`analysis-status-banner panel-feedback panel-feedback-${variant} analysis-status-banner--${variant}`}
      data-testid={testId || undefined}
    >
      {children}
    </div>
  );
}

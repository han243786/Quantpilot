import { StrategyCardNote } from "./strategySharedComponents";

export function renderFieldInput(field, value, onChange) {
  if (field.type === "select") {
    return (
      <select
        data-testid={`prop-input-${field.key || field.label}`}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      >
        {field.options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    );
  }

  if (field.type === "boolean") {
    return (
      <input
        data-testid={`prop-input-${field.key || field.label}`}
        type="checkbox"
        checked={Boolean(value)}
        onChange={(event) => onChange(event.target.checked)}
      />
    );
  }

  return (
    <input
      data-testid={`prop-input-${field.key || field.label}`}
      type={field.type === "number" ? "number" : "text"}
      value={value ?? ""}
      onChange={(event) =>
        onChange(field.type === "number" ? Number(event.target.value) : event.target.value)
      }
    />
  );
}

export function StatusChip({ tone, children }) {
  return <span className={`status-pill ${tone}`}>{children}</span>;
}

export function PropertySection({ kicker, title, summary, children, testId = null }) {
  return (
    <section
      className="property-section"
      aria-label={title}
      data-testid={testId || undefined}
    >
      <div className="property-section-header">
        {kicker ? <div className="property-section-kicker">{kicker}</div> : null}
        <div className="property-section-title">{title}</div>
        {summary ? <div className="property-section-summary">{summary}</div> : null}
      </div>
      <div className="property-section-body">{children}</div>
    </section>
  );
}

export function PropertySubsection({ title, children, testId = null }) {
  return (
    <div className="property-subsection" data-testid={testId || undefined}>
      <div className="property-subsection-title">{title}</div>
      <div className="property-subsection-body">{children}</div>
    </div>
  );
}

export function FieldGroup({ title, summary, children }) {
  return (
    <div className="property-field-group">
      <div className="property-field-group__header">
        <div className="property-field-group__title">{title}</div>
        {summary ? <div className="property-field-group__summary">{summary}</div> : null}
      </div>
      <div className="property-field-group__body">{children}</div>
    </div>
  );
}

export function PropertyPanelShell({ title, subtitle, children, className = "" }) {
  return (
    <aside className={`property-panel ${className}`.trim()}>
      <div className="property-panel-intro">
        <div className="panel-title strategy-card-title-note">
          <StrategyCardNote label={title} note={subtitle} />
        </div>
      </div>
      <div className="property-panel-scroll">{children}</div>
    </aside>
  );
}

export function WorkspaceInspectorShell({
  title,
  subtitle,
  summaryItems = [],
  actions = null,
  contextNotice = null,
  children
}) {
  return (
    <section className="workspace-section-card workspace-inspector-shell">
      <div className="workspace-section-card__header">
        <div>
          <div className="panel-title strategy-card-title-note">
            <StrategyCardNote label={title} note={subtitle} />
          </div>
        </div>
      </div>
      {contextNotice ? (
        <div className="workspace-inspector-shell__context">{contextNotice}</div>
      ) : null}
      {summaryItems.length > 0 || actions ? (
        <div className="workspace-inspector-shell__meta">
          {summaryItems.length > 0 ? (
            <div className="workspace-inspector-shell__summary">
              {summaryItems.map((item) => (
                <div
                  key={item.label}
                  className={`workspace-inspector-shell__summary-card${
                    item.tone ? ` workspace-inspector-shell__summary-card--${item.tone}` : ""
                  }`}
                >
                  <span>{item.label}</span>
                  <strong>{item.value}</strong>
                  {item.note ? <small>{item.note}</small> : null}
                </div>
              ))}
            </div>
          ) : null}
          {actions ? <div className="workspace-inspector-shell__actions">{actions}</div> : null}
        </div>
      ) : null}
      <div className="workspace-section-card__body">{children}</div>
    </section>
  );
}

import { useState } from "react";

export default function StrategyHubTemplateLibrarySection({ model }) {
  const [activeTemplateId, setActiveTemplateId] = useState("");
  const [error, setError] = useState("");
  const templates = Array.isArray(model.templateLibrary) ? model.templateLibrary : [];

  async function handleApplyTemplate(templateId) {
    setActiveTemplateId(templateId);
    setError("");
    try {
      await model.applyTemplate(templateId);
    } catch (loadError) {
      setError(loadError instanceof Error ? loadError.message : "Failed to load the selected template.");
      setActiveTemplateId("");
    }
  }

  return (
    <section
      className="strategy-template-library strategy-activity-card"
      data-testid="strategy-template-library"
    >
      <div className="strategy-card-header">
        <div>
          <div className="panel-title">Strategy template library</div>
          <div className="strategy-card-subtitle">
            Load a starter graph into the current draft without creating a second template transport.
          </div>
        </div>
      </div>

      {error ? <div className="history-note history-note-warning">{error}</div> : null}

      <div className="strategy-template-grid">
        {templates.map((template) => {
          const isLoading = activeTemplateId === template.id;
          return (
            <article
              key={template.id}
              className="strategy-template-card"
              data-testid={`strategy-template-card-${template.id}`}
            >
              <div className="strategy-template-card__header">
                <div>
                  <div className="strategy-hub-kicker">{template.category}</div>
                  <strong>{template.title}</strong>
                </div>
                <span className="status-pill info">{template.symbols.join(", ")}</span>
              </div>
              <p>{template.description}</p>
              <div className="strategy-template-meta">
                <span>{template.supportedModules.length} modules</span>
                <span>{template.symbols.length} symbols</span>
              </div>
              <button
                type="button"
                className="primary-btn"
                data-testid={`strategy-template-load-${template.id}`}
                disabled={Boolean(activeTemplateId)}
                onClick={() => void handleApplyTemplate(template.id)}
              >
                {isLoading ? "Loading template..." : "Load into draft"}
              </button>
            </article>
          );
        })}
      </div>
    </section>
  );
}

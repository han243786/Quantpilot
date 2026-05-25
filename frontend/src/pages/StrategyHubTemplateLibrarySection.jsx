import { useState } from "react";
import { StrategyCardNote } from "./StrategyHubSharedComponents";

const TEMPLATE_LIBRARY_NOTE =
  "将起始策略图加载到当前草稿，不创建第二套模板传输流程。";

export default function StrategyHubTemplateLibrarySection({ model }) {
  const [activeTemplateId, setActiveTemplateId] = useState("");
  const [error, setError] = useState("");
  // v3.6.0 U5: 首次访问默认展开模板库
  const [isExpanded, setIsExpanded] = useState(() => {
    if (localStorage.getItem("quantpilot_template_visited")) return false;
    localStorage.setItem("quantpilot_template_visited", "1");
    return true;
  });
  const templates = Array.isArray(model.templateLibrary) ? model.templateLibrary : [];

  async function handleApplyTemplate(templateId) {
    setActiveTemplateId(templateId);
    setError("");
    try {
      await model.applyTemplate(templateId);
    } catch (loadError) {
      setError(loadError instanceof Error ? loadError.message : "加载所选模板失败。");
      setActiveTemplateId("");
    }
  }

  return (
    <section
      className={`strategy-template-library strategy-activity-card${
        isExpanded ? " strategy-template-library--expanded" : " strategy-template-library--collapsed"
      }`}
      data-testid="strategy-template-library"
    >
      <div className="strategy-card-header">
        <div>
          <div className="panel-title strategy-card-title-note">
            <StrategyCardNote label="策略模板库" note={TEMPLATE_LIBRARY_NOTE} />
          </div>
        </div>
        <button
          type="button"
          className="ghost-btn strategy-template-library__toggle"
          aria-controls="strategy-template-library-grid"
          aria-expanded={isExpanded}
          data-testid="strategy-template-library-toggle"
          onClick={() => setIsExpanded((current) => !current)}
        >
          <span aria-hidden="true">{isExpanded ? "-" : "+"}</span>
          {isExpanded ? "收起模板" : "展开模板"}
        </button>
      </div>

      {error ? <div className="history-note history-note-warning">{error}</div> : null}

      {isExpanded ? (
        <div className="strategy-template-grid" id="strategy-template-library-grid">
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
                  <div style={{ display: "flex", gap: 6, flexWrap: "wrap", justifyContent: "flex-end" }}>
                    {template.runtimeVersion ? (
                      <span className="status-pill warning">{template.runtimeVersion}</span>
                    ) : null}
                    <span className="status-pill info">{template.symbols.join(", ")}</span>
                  </div>
                </div>
                <p>{template.description}</p>
                <div className="strategy-template-meta">
                  <span>{template.supportedModules.length} 个模块</span>
                  <span>{template.symbols.length} 个标的</span>
                </div>
                <button
                  type="button"
                  className="primary-btn"
                  data-testid={`strategy-template-load-${template.id}`}
                  disabled={Boolean(activeTemplateId)}
                  onClick={() => void handleApplyTemplate(template.id)}
                >
                  {isLoading ? "正在加载模板..." : "加载到草稿"}
                </button>
              </article>
            );
          })}
        </div>
      ) : null}
    </section>
  );
}

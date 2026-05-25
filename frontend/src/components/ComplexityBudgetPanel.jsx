function formatNumber(value) {
  return Number(value || 0).toLocaleString("zh-CN");
}

const DEFAULT_LIMITS = {
  state_count: 1024,
  transition_count: 2048,
  memory_field_count: 1024,
  nested_machine_depth: 2,
  event_processing_path_count: 4096
};

const ROWS = [
  ["state_count", "状态总数"],
  ["transition_count", "迁移总数"],
  ["memory_field_count", "内存字段"],
  ["nested_machine_depth", "嵌套深度"],
  ["event_processing_path_count", "事件路径"]
];

export default function ComplexityBudgetPanel({
  metrics = {},
  limits = DEFAULT_LIMITS,
  testId = "complexity-budget-panel"
}) {
  const rows = ROWS.map(([key, label]) => {
    const value = Number(metrics?.[key]) || 0;
    const limit = Number(limits?.[key] || DEFAULT_LIMITS[key]) || 1;
    const ratio = value / limit;
    const tone = ratio > 1 ? "danger" : ratio >= 0.8 ? "warning" : "neutral";
    return { key, label, value, limit, ratio, tone };
  });

  return (
    <div className="mini-list" data-testid={testId}>
      <div className="mini-list-title">复杂度预算</div>
      {rows.map((row) => (
        <div key={row.key} className="mini-item" data-testid={`${testId}-${row.key}`}>
          <div className="kv-line">
            <span>{row.label}</span>
            <strong>
              {formatNumber(row.value)} / {formatNumber(row.limit)}
            </strong>
          </div>
          <div className="muted-line">
            <span className={`status-pill ${row.tone}`}>
              {row.ratio > 1 ? "超预算" : row.ratio >= 0.8 ? "接近上限" : "正常"}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}

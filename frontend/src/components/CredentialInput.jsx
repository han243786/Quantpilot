import { useState, useCallback, useEffect } from "react";
import { useI18n } from "../i18n";

/**
 * 解析用户粘贴的凭证明文为字段 Map。
 * 支持格式:
 *   JSON:  {"api_key": "...", "secret": "...", "passphrase": "..."}
 *   KV:    api_key=...\nsecret=...\npassphrase=...
 */
function parseCredentialBlock(input) {
  const trimmed = input.trim();
  if (!trimmed) return null;

  // 尝试 JSON
  if (trimmed.startsWith("{")) {
    try {
      const parsed = JSON.parse(trimmed);
      if (typeof parsed === "object" && !Array.isArray(parsed)) {
        const map = {};
        Object.entries(parsed).forEach(([k, v]) => {
          if (typeof v === "string" && v.trim()) map[k] = v.trim();
        });
        if (Object.keys(map).length > 0) return map;
      }
    } catch (_) { /* 非 JSON, 继续尝试 KV */ }
  }

  // 尝试 key=value 逐行
  const lines = trimmed.split(/\r?\n/).filter((l) => l.trim());
  const map = {};
  for (const line of lines) {
    const eq = line.indexOf("=");
    if (eq > 0) {
      const key = line.substring(0, eq).trim();
      const value = line.substring(eq + 1).trim();
      if (key && value) map[key] = value;
    }
  }
  if (Object.keys(map).length > 0) return map;

  return null;
}

export default function CredentialInput({
  label,
  initialValues = {},
  onSave,
  onDelete,
  onCancel,
}) {
  const { t } = useI18n();
  const [rawInput, setRawInput] = useState("");
  const [parsedFields, setParsedFields] = useState(null);
  const [parseError, setParseError] = useState("");
  const [saving, setSaving] = useState(false);
  const [showPreview, setShowPreview] = useState(false);

  // 卸载时清零
  useEffect(() => {
    return () => {
      setRawInput("");
      setParsedFields(null);
      setSaving(false);
    };
  }, []);

  const handleInputChange = useCallback((e) => {
    const value = e.target.value;
    setRawInput(value);
    const result = parseCredentialBlock(value);
    if (result) {
      setParsedFields(result);
      setParseError("");
    } else if (value.trim()) {
      setParsedFields(null);
      setParseError(t("无法解析凭证，请粘贴 JSON 或 key=value 格式"));
    } else {
      setParsedFields(null);
      setParseError("");
    }
  }, [t]);

  const [saveError, setSaveError] = useState("");

  const handleSave = useCallback(async () => {
    if (!onSave || !parsedFields) return;
    setSaving(true);
    setSaveError("");
    try {
      await onSave(label, { ...parsedFields });
    } catch (e) {
      setSaveError(e?.message || t("保存凭证失败"));
    } finally {
      setSaving(false);
    }
  }, [label, parsedFields, onSave]);

  const handleCancel = useCallback(() => {
    setRawInput("");
    setParsedFields(null);
    onCancel?.();
  }, [onCancel]);

  const fieldCount = parsedFields ? Object.keys(parsedFields).length : 0;

  return (
    <div className="credential-form" data-testid="credential-form">
      {label ? (
        <div className="credential-form-label">{label}</div>
      ) : null}

      <label className="credential-field">
        <span>{t("粘贴凭证")}</span>
        <textarea
          autoComplete="off"
          spellCheck={false}
          rows={6}
          value={rawInput}
          onChange={handleInputChange}
          placeholder={t("粘贴交易所提供的完整凭证内容\n支持 JSON 格式: {\"api_key\":\"...\",\"secret\":\"...\"}\n或 key=value 格式: api_key=...\\nsecret=...")}
          data-testid="credential-input-block"
        />
      </label>

      {parseError ? (
        <div className="panel-feedback panel-feedback-warning" style={{ marginTop: 8 }}>
          {parseError}
        </div>
      ) : null}

      {parsedFields && rawInput.trim() ? (
        <div style={{ marginTop: 8 }}>
          <button
            className="ad-btn ad-btn--ghost compact-btn"
            onClick={() => setShowPreview(!showPreview)}
            data-testid="credential-toggle-preview"
          >
            {showPreview ? t("隐藏字段预览") : t("已解析 {count} 个字段，点击预览", { count: fieldCount })}
          </button>
          {showPreview ? (
            <ul className="credential-list" style={{ marginTop: 8 }}>
              {Object.entries(parsedFields).map(([key, value]) => (
                <li key={key} className="credential-list-item">
                  <span>{key}</span>
                  <span className="muted-line">{value.substring(0, 4)}****</span>
                </li>
              ))}
            </ul>
          ) : null}
        </div>
      ) : null}

      {saveError ? (
        <div className="panel-feedback panel-feedback-error" style={{ marginTop: 8 }}>
          {saveError}
        </div>
      ) : null}

      <div className="credential-form-actions" style={{ marginTop: 12 }}>
        {onSave ? (
          <button
            className="ad-btn ad-btn--primary"
            onClick={handleSave}
            disabled={!parsedFields || saving}
            data-testid="credential-save"
          >
            {saving ? t("保存中...") : t("保存")}
          </button>
        ) : null}
        {onDelete ? (
          <button
            className="ad-btn ad-btn--ghost"
            onClick={() => onDelete(label)}
            data-testid="credential-delete"
          >
            {t("删除")}
          </button>
        ) : null}
        {onCancel ? (
          <button
            className="ad-btn ad-btn--ghost"
            onClick={handleCancel}
            data-testid="credential-cancel"
          >
            {t("取消")}
          </button>
        ) : null}
      </div>
    </div>
  );
}

/**
 * 便捷组件：OKX 交易所预设
 */
export function OkxCredentialInput(props) {
  return <CredentialInput {...props} />;
}

import { useState, useCallback, useEffect } from "react";
import { useI18n } from "../i18n";

export default function CredentialInput({
  label,
  fields = [],
  initialValues = {},
  onSave,
  onDelete,
  onCancel,
}) {
  const { t } = useI18n();
  const [values, setValues] = useState(() => {
    const init = {};
    fields.forEach((f) => {
      init[f.name] = initialValues[f.name] || "";
    });
    return init;
  });
  const [saving, setSaving] = useState(false);

  // 组件卸载时主动清零 state，防止凭证明文残留在 React 内存中
  useEffect(() => {
    return () => {
      setValues({});
      setSaving(false);
    };
  }, []);

  const updateField = useCallback((name, value) => {
    setValues((prev) => ({ ...prev, [name]: value }));
  }, []);

  const handleSave = useCallback(async () => {
    if (!onSave) return;
    setSaving(true);
    try {
      await onSave(label, { ...values });
    } finally {
      setSaving(false);
    }
  }, [label, values, onSave]);

  const handleCancel = useCallback(() => {
    setValues({});
    onCancel?.();
  }, [onCancel]);

  const allRequiredFilled = fields.every(
    (f) => !f.required || (values[f.name] || "").trim()
  );

  return (
    <div className="credential-form" data-testid="credential-form">
      {label ? (
        <div className="credential-form-label">{label}</div>
      ) : null}

      {fields.map((field) => (
        <label className="credential-field" key={field.name}>
          <span>{field.label || field.name}</span>
          <input
            type="password"
            autoComplete="off"
            spellCheck={false}
            value={values[field.name] || ""}
            onChange={(e) => updateField(field.name, e.target.value)}
            placeholder={field.placeholder || t("请输入")}
            required={field.required}
            data-testid={`credential-input-${field.name}`}
          />
        </label>
      ))}

      <div className="credential-form-actions">
        {onSave ? (
          <button
            className="primary-btn"
            onClick={handleSave}
            disabled={!allRequiredFilled || saving}
            data-testid="credential-save"
          >
            {saving ? t("保存中...") : t("保存")}
          </button>
        ) : null}
        {onDelete ? (
          <button
            className="ghost-btn"
            onClick={() => onDelete(label)}
            data-testid="credential-delete"
          >
            {t("删除")}
          </button>
        ) : null}
        {onCancel ? (
          <button
            className="ghost-btn"
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
 * 便捷组件：为 OKX 交易所预定义三个字段
 */
export function OkxCredentialInput(props) {
  return (
    <CredentialInput
      fields={[
        { name: "key", label: "API Key", required: true },
        { name: "secret", label: "Secret Key", required: true },
        { name: "passphrase", label: "通行密钥", required: true },
      ]}
      {...props}
    />
  );
}

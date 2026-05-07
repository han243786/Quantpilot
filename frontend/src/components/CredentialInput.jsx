export default function CredentialInput({ label, value, onChange, placeholder }) {
  return (
    <label className="credential-field">
      <span>{label}</span>
      <input
        type="password"
        autoComplete="off"
        spellCheck={false}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder || "请输入"}
        data-testid={`credential-input-${label}`}
      />
    </label>
  );
}

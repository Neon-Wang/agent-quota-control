interface ToggleSwitchProps {
  checked: boolean;
  disabled?: boolean;
  "aria-label": string;
  onChange: (checked: boolean) => void;
}

export function ToggleSwitch({
  checked,
  disabled = false,
  onChange,
  "aria-label": ariaLabel,
}: ToggleSwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={ariaLabel}
      className={checked ? "toggle-switch on" : "toggle-switch"}
      disabled={disabled}
      onClick={() => onChange(!checked)}
    >
      <span className="toggle-knob" aria-hidden />
    </button>
  );
}

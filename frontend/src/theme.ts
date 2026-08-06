export type ThemePreference = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

export const THEME_STORAGE_KEY = "agent-quota-control.theme";

const themeListeners = new Set<(preference: ThemePreference) => void>();

export function readThemePreference(): ThemePreference {
  try {
    const value = localStorage.getItem(THEME_STORAGE_KEY);
    if (value === "light" || value === "dark" || value === "system") {
      return value;
    }
  } catch {
    // Ignore unavailable storage (private mode / tests).
  }
  return "system";
}

export function resolveTheme(preference: ThemePreference): ResolvedTheme {
  if (preference === "light" || preference === "dark") {
    return preference;
  }
  try {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  } catch {
    return "light";
  }
}

export function applyResolvedTheme(resolved: ResolvedTheme) {
  document.documentElement.dataset.theme = resolved;
  document.documentElement.style.colorScheme = resolved;
}

export function applyThemePreference(preference: ThemePreference) {
  applyResolvedTheme(resolveTheme(preference));
}

export function setThemePreference(preference: ThemePreference) {
  try {
    localStorage.setItem(THEME_STORAGE_KEY, preference);
  } catch {
    // Ignore unavailable storage.
  }
  applyThemePreference(preference);
  for (const listener of themeListeners) {
    listener(preference);
  }
}

export function subscribeThemePreference(
  listener: (preference: ThemePreference) => void,
) {
  themeListeners.add(listener);
  return () => {
    themeListeners.delete(listener);
  };
}

export function subscribeSystemTheme(
  preference: ThemePreference,
  onChange: () => void,
) {
  if (preference !== "system" || typeof window.matchMedia !== "function") {
    return () => undefined;
  }

  const media = window.matchMedia("(prefers-color-scheme: dark)");
  const handleChange = () => onChange();
  media.addEventListener("change", handleChange);
  return () => media.removeEventListener("change", handleChange);
}

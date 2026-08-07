export type Locale = "en" | "zh-CN";

export const LOCALES: readonly Locale[] = ["zh-CN", "en"] as const;
export const DEFAULT_LOCALE: Locale = "en";
export const FALLBACK_LOCALE: Locale = "en";
export const LOCALE_STORAGE_KEY = "agent-quota-control.locale";

const localeListeners = new Set<(locale: Locale) => void>();

export function isLocale(value: string | null | undefined): value is Locale {
  return value === "en" || value === "zh-CN";
}

export function detectLocale(): Locale {
  try {
    const stored = localStorage.getItem(LOCALE_STORAGE_KEY);
    if (isLocale(stored)) return stored;
  } catch {
    // Ignore unavailable storage.
  }

  try {
    const language = navigator.language?.toLowerCase() ?? "";
    if (language.startsWith("zh")) return "zh-CN";
  } catch {
    // Ignore unavailable navigator.
  }

  return DEFAULT_LOCALE;
}

export function readLocale(): Locale {
  return detectLocale();
}

export function setLocale(locale: Locale) {
  try {
    localStorage.setItem(LOCALE_STORAGE_KEY, locale);
  } catch {
    // Ignore unavailable storage.
  }
  try {
    document.documentElement.lang = locale === "zh-CN" ? "zh-CN" : "en";
  } catch {
    // Ignore missing document in non-DOM contexts.
  }
  for (const listener of localeListeners) {
    listener(locale);
  }
}

export function subscribeLocale(listener: (locale: Locale) => void) {
  localeListeners.add(listener);
  return () => {
    localeListeners.delete(listener);
  };
}

export function applyDocumentLocale(locale: Locale) {
  try {
    document.documentElement.lang = locale === "zh-CN" ? "zh-CN" : "en";
  } catch {
    // Ignore missing document.
  }
}

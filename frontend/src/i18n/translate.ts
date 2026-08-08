import { formatMessage, type TranslateValues } from "./format";
import type { Locale } from "./locales";
import type { MessageCatalog, MessageNamespace } from "./message-catalog";
import en from "./messages/en.json";
import zhCN from "./messages/zh-CN.json";
import { FALLBACK_LOCALE } from "./locales";

const catalogs: Record<Locale, MessageCatalog> = {
  en: en as MessageCatalog,
  "zh-CN": zhCN as MessageCatalog,
};

export function getCatalog(locale: Locale): MessageCatalog {
  return catalogs[locale] ?? catalogs[FALLBACK_LOCALE];
}

export function lookupMessage(
  locale: Locale,
  namespace: MessageNamespace,
  key: string,
): string | undefined {
  const primary = getCatalog(locale)[namespace] as Record<string, string> | undefined;
  if (primary?.[key] != null) return primary[key];
  if (locale !== FALLBACK_LOCALE) {
    const fallback = getCatalog(FALLBACK_LOCALE)[namespace] as
      | Record<string, string>
      | undefined;
    return fallback?.[key];
  }
  return undefined;
}

export function translate(
  locale: Locale,
  namespace: MessageNamespace,
  key: string,
  values?: TranslateValues,
): string {
  const template = lookupMessage(locale, namespace, key);
  if (template == null) {
    if (import.meta.env.DEV) {
      console.warn(`[i18n] missing ${namespace}.${key} for ${locale}`);
    }
    return `${namespace}.${key}`;
  }
  return formatMessage(template, values);
}

export type Translator<N extends MessageNamespace> = (
  key: Extract<keyof MessageCatalog[N], string>,
  values?: TranslateValues,
) => string;

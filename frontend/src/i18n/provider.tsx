import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { createFormatter } from "./format";
import {
  applyDocumentLocale,
  readLocale,
  setLocale as persistLocale,
  subscribeLocale,
  type Locale,
} from "./locales";
import type { MessageCatalog, MessageNamespace } from "./message-catalog";
import {
  getCatalog,
  translate,
  type Translator,
} from "./translate";

interface I18nContextValue {
  locale: Locale;
  messages: MessageCatalog;
  setLocale: (locale: Locale) => void;
  t: <N extends MessageNamespace>(
    namespace: N,
  ) => Translator<N>;
  format: ReturnType<typeof createFormatter>;
}

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(() => readLocale());

  useEffect(() => {
    applyDocumentLocale(locale);
    return subscribeLocale((next) => {
      setLocaleState(next);
      applyDocumentLocale(next);
    });
  }, [locale]);

  const value = useMemo<I18nContextValue>(() => {
    return {
      locale,
      messages: getCatalog(locale),
      setLocale: (next) => {
        persistLocale(next);
        setLocaleState(next);
      },
      t: (namespace) => (key, values) =>
        translate(locale, namespace, key, values),
      format: createFormatter(locale === "zh-CN" ? "zh-CN" : "en-US"),
    };
  }, [locale]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18nContext(): I18nContextValue {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n hooks require <I18nProvider>");
  }
  return context;
}

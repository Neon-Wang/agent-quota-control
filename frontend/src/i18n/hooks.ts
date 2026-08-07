import type { MessageNamespace } from "./message-catalog";
import { useI18nContext } from "./provider";
import type { Translator } from "./translate";

export function useLocale() {
  const { locale, setLocale } = useI18nContext();
  return { locale, setLocale };
}

export function useTranslations<N extends MessageNamespace>(
  namespace: N,
): Translator<N> {
  const { t } = useI18nContext();
  return t(namespace);
}

export function useFormatter() {
  return useI18nContext().format;
}

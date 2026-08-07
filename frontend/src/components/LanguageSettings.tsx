import { ChevronDown, Languages } from "lucide-react";
import { useLocale, useTranslations, type Locale } from "../i18n";

const localeOptions: Array<{ id: Locale; labelKey: "locale_zh_cn" | "locale_en" }> = [
  { id: "zh-CN", labelKey: "locale_zh_cn" },
  { id: "en", labelKey: "locale_en" },
];

export function LanguageSettings() {
  const t = useTranslations("settings");
  const { locale, setLocale } = useLocale();

  return (
    <section className="panel">
      <div className="panel-title">
        <Languages size={15} strokeWidth={1.75} aria-hidden />
        {t("language")}
      </div>
      <div className="macos-popup">
        <select
          value={locale}
          aria-label={t("language")}
          onChange={(event) => setLocale(event.currentTarget.value as Locale)}
        >
          {localeOptions.map((option) => (
            <option key={option.id} value={option.id}>
              {t(option.labelKey)}
            </option>
          ))}
        </select>
        <ChevronDown className="macos-popup-chevron" size={14} strokeWidth={2} aria-hidden />
      </div>
    </section>
  );
}

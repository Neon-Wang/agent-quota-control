import { render, type RenderOptions } from "@testing-library/react";
import type { ReactElement, ReactNode } from "react";
import { I18nProvider, LOCALE_STORAGE_KEY, setLocale } from "../i18n";

export function renderWithI18n(
  ui: ReactElement,
  options?: Omit<RenderOptions, "wrapper">,
) {
  localStorage.setItem(LOCALE_STORAGE_KEY, "zh-CN");
  setLocale("zh-CN");

  function Wrapper({ children }: { children: ReactNode }) {
    return <I18nProvider>{children}</I18nProvider>;
  }

  return render(ui, { wrapper: Wrapper, ...options });
}

import "@testing-library/jest-dom";
import { beforeEach } from "vitest";
import { LOCALE_STORAGE_KEY, setLocale } from "./i18n";

beforeEach(() => {
  localStorage.setItem(LOCALE_STORAGE_KEY, "zh-CN");
  setLocale("zh-CN");
});

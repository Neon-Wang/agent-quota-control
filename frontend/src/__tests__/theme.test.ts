import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  applyThemePreference,
  readThemePreference,
  resolveTheme,
  setThemePreference,
  THEME_STORAGE_KEY,
} from "../theme";

describe("theme preference", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
    document.documentElement.style.colorScheme = "";
  });

  afterEach(() => {
    localStorage.clear();
    vi.restoreAllMocks();
  });

  it("defaults to system and resolves from the OS preference", () => {
    vi.spyOn(window, "matchMedia").mockImplementation((query) => {
      return {
        matches: query.includes("dark"),
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      } as MediaQueryList;
    });

    expect(readThemePreference()).toBe("system");
    expect(resolveTheme("system")).toBe("dark");
    expect(resolveTheme("light")).toBe("light");
  });

  it("persists an explicit theme and applies it to the document", () => {
    setThemePreference("light");
    expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe("light");
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(document.documentElement.style.colorScheme).toBe("light");

    setThemePreference("dark");
    expect(readThemePreference()).toBe("dark");
    applyThemePreference("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
  });
});

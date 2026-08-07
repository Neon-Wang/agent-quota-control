import { SunMoon } from "lucide-react";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTranslations } from "../i18n";
import {
  applyThemePreference,
  readThemePreference,
  setThemePreference,
  subscribeSystemTheme,
  type ThemePreference,
} from "../theme";

const themeOptions: Array<{
  id: ThemePreference;
  labelKey: "theme_light" | "theme_dark" | "theme_system";
}> = [
  { id: "light", labelKey: "theme_light" },
  { id: "dark", labelKey: "theme_dark" },
  { id: "system", labelKey: "theme_system" },
];

export function AppearanceSettings() {
  const t = useTranslations("settings");
  const [preference, setPreference] = useState<ThemePreference>(() =>
    readThemePreference(),
  );
  const [pressedTheme, setPressedTheme] = useState<ThemePreference | null>(null);

  useEffect(() => {
    applyThemePreference(preference);
    void syncNativeTheme(preference);
    return subscribeSystemTheme(preference, () => {
      applyThemePreference(preference);
      void syncNativeTheme(preference);
    });
  }, [preference]);

  function selectTheme(next: ThemePreference) {
    setPreference(next);
    setThemePreference(next);
    void syncNativeTheme(next);
  }

  return (
    <section className="panel">
      <div className="panel-title">
        <SunMoon size={15} strokeWidth={1.75} aria-hidden />
        {t("appearance")}
      </div>
      <div className="segmented" role="group" aria-label={t("appearance_mode")}>
        {themeOptions.map((option) => (
          <button
            key={option.id}
            type="button"
            className={
              pressedTheme === option.id ||
              (pressedTheme === null && preference === option.id)
                ? "active"
                : ""
            }
            aria-pressed={preference === option.id}
            onPointerDown={() => setPressedTheme(option.id)}
            onPointerLeave={() => {
              if (pressedTheme === option.id) setPressedTheme(null);
            }}
            onPointerCancel={() => setPressedTheme(null)}
            onContextMenu={() => setPressedTheme(null)}
            onClick={() => {
              selectTheme(option.id);
              setPressedTheme(null);
            }}
          >
            {t(option.labelKey)}
          </button>
        ))}
      </div>
    </section>
  );
}

async function syncNativeTheme(preference: ThemePreference) {
  try {
    await getCurrentWindow().setTheme(
      preference === "system" ? null : preference,
    );
  } catch {
    // Theme API may be unavailable in tests or restricted runtimes.
  }
}

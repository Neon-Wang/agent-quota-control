import { SunMoon } from "lucide-react";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  applyThemePreference,
  readThemePreference,
  setThemePreference,
  subscribeSystemTheme,
  type ThemePreference,
} from "../theme";

const themeOptions: Array<{ id: ThemePreference; label: string }> = [
  { id: "light", label: "浅色" },
  { id: "dark", label: "深色" },
  { id: "system", label: "跟随系统" },
];

export function AppearanceSettings() {
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
        外观
      </div>
      <div className="segmented" role="group" aria-label="外观模式">
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
            {option.label}
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

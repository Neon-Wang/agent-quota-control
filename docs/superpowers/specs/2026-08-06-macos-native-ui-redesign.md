# macOS Native UI Redesign

Date: 2026-08-06  
Branch: `ui/macos-native-redesign`  
Fork: `Zhaohan-Wang/agent-quota-control` → PR back to `Neon-Wang/agent-quota-control`

## Goal

Make the Tauri dashboard feel like a first-party macOS utility: quiet, hierarchical,
system-aware, and consistent with apps such as System Settings, Activity Monitor,
and Battery settings — not a generic SaaS admin panel.

## Can We Call UIKit?

**No — not in a useful way for this app.**

| Layer | Reality |
| --- | --- |
| Platform | This is a **macOS** app. UIKit is iOS/iPadOS. The desktop counterpart is **AppKit** / **SwiftUI**. |
| Dashboard | Rendered inside **WKWebView** (Tauri). React/CSS cannot instantiate `NSButton`, `NSSwitch`, or SwiftUI views directly. |
| Existing native code | Tray, Keychain, WidgetKit, and reload helper already use AppKit/Swift. The dashboard itself stays web. |

### Practical options

1. **Recommended (this redesign):** Keep React + CSS, but match Apple HIG tokens:
   SF Pro / `-apple-system`, semantic light/dark colors, inset grouped panels,
   native-looking segmented controls and switches, quieter typography.
2. **Optional polish (phase 2):** Use AppKit via Rust (`window-vibrancy` /
   `NSVisualEffectView`) for sidebar translucency. Needs transparent window +
   careful WKWebView background clearing; higher risk, do after visual tokens land.
3. **Not recommended now:** Rewrite the whole dashboard in SwiftUI. Massive rewrite,
   duplicates existing React tests/logic, and blocks the current Tauri IPC surface.

Conclusion: improve the WebView UI to *look* native; call AppKit only where the
shell needs real materials (vibrancy), not by “dropping in UIKit”.

## What’s Wrong Today

### 1. No appearance system
- Single warm off-white palette (`#f7f7f5`, `#f0f0ed`).
- Dozens of hardcoded light-only hex values (`#fafafa`, `#f5f7fa`, `#181818` button).
- No `prefers-color-scheme` / dark tokens → fails the most basic macOS expectation.

### 2. Wrong visual language
- Flat “admin card” surfaces with hard borders, little depth or material layering.
- Primary button is solid black — not macOS push-button / tinted control language.
- Checkboxes for settings that should be **toggle switches**.
- Segmented control uses a light-blue fill that reads iOS-ish / custom web, not
  `NSSegmentedControl`.
- Lucide icons instead of SF Symbol weight/optical sizing (acceptable in WebView,
  but strokes feel heavier than system icons).

### 3. Broken information hierarchy
- Giant status words (“不够” / “够”) dominate the card; Apple utilities lead with
  the **metric**, then a compact status badge.
- Cards force `min-height: 520px`, creating empty vertical waste.
- Proxy / update metadata compete with the forecast; footer feels like form noise.
- Topbar proxy pills + black refresh fight the title for attention.

### 4. Sidebar is not a macOS sidebar
- Opaque warm gray, no vibrancy / separation.
- Selection is neutral gray chip; system sidebars use accent-tinted selection when
  key window is active.
- Brand block repeats window title chrome and crowds traffic lights.

### 5. Settings density and grouping
- Monitoring + accounts dumped into a 2-column card grid.
- Account form is a dense multi-column strip that wraps poorly.
- Switch rows use oversized 18px labels and raw checkboxes — not Settings list rows.

### 6. Charts / meters not theme-aware
- Progress track `#e7ebef`, chart strokes, and pending boxes are light-only.
- Severity greens/oranges are OK, but need dark-mode counterparts and calmer fills.

### 7. Motion / feedback
- Almost no state transitions; native apps use short, restrained opacity/transform.
- Loading state is a bare centered spinner with no window chrome.

## Target Aesthetic (reference apps)

- **System Settings:** inset grouped lists, trailing toggles, quiet section headers.
- **Activity Monitor / Battery:** clear meters, compact status, charts secondary.
- **SF Symbols + SF Pro:** regular/semibold, not ultra-heavy display type for status.
- **Appearance:** follow system light/dark automatically; optional later “match accent”.

## Design Tokens

```text
Light                          Dark
window bg    #F2F2F7           #1C1C1E
sidebar      #EBEBF0 / blur    #2C2C2E / blur
content      #F2F2F7           #1C1C1E
grouped      #FFFFFF           #2C2C2E
separator    rgba(60,60,67,.12) rgba(84,84,88,.65)
label        #000000           #FFFFFF
secondary    #3C3C43 @ 60%     #EBEBF5 @ 60%
accent       system blue       system blue
ok / warn / danger             slightly desaturated for dark
```

Typography:
- UI: `-apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", sans-serif`
- Title 3 / Headline / Subheadline / Footnote scale (approx 20 / 15 / 13 / 11)
- Status word demoted from 28px display to badge or headline ≤ 17px

## Implementation Plan

### Phase A — Foundation (this PR)
1. Rebuild `styles.css` around semantic CSS variables + `@media (prefers-color-scheme)`.
2. Shell: Settings-like sidebar, quieter topbar, accent selection, macOS-like buttons.
3. Controls: CSS toggle switch, segmented control, inset grouped panels, list rows.
4. Quota cards: metric-first layout, compact status badge, themeable meters/charts.
5. Settings/Monitoring/Accounts: single-column grouped sections on wide content.
6. Keep structure/API stable so existing Vitest coverage mostly stays green.

### Phase B — Native shell polish (follow-up)
1. Evaluate `window-vibrancy` / `NSVisualEffectView` for sidebar only.
2. Clear WKWebView background when using transparency.
3. Sync window theme via Tauri `theme` APIs if needed for titlebar chrome.
4. Consider SF Symbols via SVG sprite set matching system names (optional).

### Phase C — Out of scope for UI PR
- Rewriting dashboard in SwiftUI
- Changing tray / widget visual language (separate pass)
- Localization beyond zh-CN

## Acceptance Criteria

- [ ] Light and dark modes both look intentional; switching system appearance updates UI live.
- [ ] No light-only hardcoded surfaces remain in primary screens.
- [ ] Dashboard reads as a macOS utility: sidebar + grouped content, not a web admin.
- [ ] Quota cards prioritize utilization / forecast without shouting status words.
- [ ] Settings use toggle switches and native-feeling segmented controls.
- [ ] `pnpm typecheck` and `pnpm test:frontend` pass.

## Risks

- Vibrancy + transparent Tauri windows can regress drag regions / titlebar; defer.
- Chart SVG hard-coded strokes need CSS variables or they will flash in dark mode.
- Layout tests assert `.quota-card` rules; avoid reintroducing `height: 100%`.

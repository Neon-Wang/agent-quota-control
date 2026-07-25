#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

bash scripts/probe-widget-signing.sh
bash scripts/build-macos-widget.sh
swift test --package-path macos-widget
pnpm tauri build --bundles app

app="$repo_root/src-tauri/target/release/bundle/macos/Agent Quota Control.app"
extension="$repo_root/macos-widget/build/release/AgentQuotaWidget.appex"
plugins="$app/Contents/PlugIns"

if [[ ! -d "$app" ]]; then
  echo "error: Tauri app bundle was not created at $app" >&2
  exit 1
fi

identity_line="$({ security find-identity -v -p codesigning 2>/dev/null || true; } \
  | awk '/Apple Development:/ && $0 !~ /REVOKED/ { print; exit }')"
identity="$(awk '{ print $2 }' <<<"$identity_line")"
if [[ -z "$identity" ]]; then
  echo "error: no Apple Development signing identity is available" >&2
  exit 1
fi

mkdir -p "$plugins"
ditto "$extension" "$plugins/AgentQuotaWidget.appex"

codesign --force --sign "$identity" --timestamp=none --options runtime \
  --entitlements "$repo_root/macos-widget/Sources/Widget/Widget.entitlements" \
  "$plugins/AgentQuotaWidget.appex"
codesign --force --sign "$identity" --timestamp=none --options runtime \
  "$app/Contents/MacOS/agent-quota-widget-reload"
codesign --force --sign "$identity" --timestamp=none --options runtime \
  --entitlements "$repo_root/src-tauri/Entitlements.plist" \
  "$app"

codesign --verify --deep --strict --verbose=2 "$app"
pluginkit -m -A -D -v -i io.ccswitch.agent-quota-control.widget || true

echo "signed app with widget: $app"

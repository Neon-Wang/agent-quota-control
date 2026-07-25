#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
widget_root="$repo_root/macos-widget"
sdk_path="$(xcrun --sdk macosx --show-sdk-path)"
architecture="$(uname -m)"
target_triple="${architecture}-apple-macos14.0"
case "$architecture" in
  arm64) rust_architecture="aarch64" ;;
  x86_64) rust_architecture="x86_64" ;;
  *) echo "error: unsupported architecture $architecture" >&2; exit 1 ;;
esac
rust_target="${rust_architecture}-apple-darwin"
build_root="$widget_root/build/release"
extension="$build_root/AgentQuotaWidget.appex"
extension_binary="$extension/Contents/MacOS/AgentQuotaWidget"
extension_resources="$extension/Contents/Resources"
objects_root="$build_root/Objects"
widget_object="$objects_root/AgentQuotaWidget.o"
widget_module="$objects_root/AgentQuotaWidget.swiftmodule"
const_values="$objects_root/AgentQuotaWidget.swiftconstvalues"
source_file_list="$objects_root/AgentQuotaWidget.SwiftFileList"
const_values_list="$objects_root/AgentQuotaWidget.SwiftConstValuesFileList"
protocols_file="$widget_root/AppIntentsProtocols.json"
sidecar="$repo_root/src-tauri/bin/agent-quota-widget-reload-$rust_target"
toolchain_dir="$(xcode-select -p)/Toolchains/XcodeDefault.xctoolchain"
xcode_build_version="$(xcodebuild -version | awk '/Build version/ { print $3 }')"

mkdir -p "$extension/Contents/MacOS" "$extension_resources" "$objects_root" "$(dirname "$sidecar")"
cp "$widget_root/Sources/Widget/Info.plist" "$extension/Contents/Info.plist"
find "$widget_root/Sources/Shared" "$widget_root/Sources/Configuration" "$widget_root/Sources/Widget" \
  -type f -name '*.swift' -print | sort > "$source_file_list"

xcrun swiftc \
  -swift-version 5 \
  -parse-as-library \
  -application-extension \
  -O \
  -whole-module-optimization \
  -emit-module \
  -emit-module-path "$widget_module" \
  -c \
  -target "$target_triple" \
  -sdk "$sdk_path" \
  -framework SwiftUI \
  -framework WidgetKit \
  -framework AppIntents \
  -module-name AgentQuotaWidget \
  -Xfrontend -const-gather-protocols-file \
  -Xfrontend "$protocols_file" \
  -emit-const-values-path "$const_values" \
  "$widget_root"/Sources/Shared/*.swift \
  "$widget_root"/Sources/Configuration/*.swift \
  "$widget_root"/Sources/Widget/*.swift \
  -o "$widget_object"

xcrun swiftc \
  -target "$target_triple" \
  -sdk "$sdk_path" \
  -application-extension \
  -framework SwiftUI \
  -framework WidgetKit \
  -framework AppIntents \
  "$widget_object" \
  -Xlinker -e \
  -Xlinker _NSExtensionMain \
  -o "$extension_binary"

if ! nm "$extension_binary" | grep ' U _NSExtensionMain$' >/dev/null; then
  echo "error: Widget extension is not linked through NSExtensionMain" >&2
  exit 1
fi

echo "$const_values" > "$const_values_list"
xcrun appintentsmetadataprocessor \
  --output "$extension_resources" \
  --toolchain-dir "$toolchain_dir" \
  --module-name AgentQuotaWidget \
  --sdk-root "$sdk_path" \
  --xcode-version "$xcode_build_version" \
  --platform-family macOS \
  --deployment-target 14.0 \
  --target-triple "$target_triple" \
  --source-file-list "$source_file_list" \
  --swift-const-vals-list "$const_values_list" \
  --binary-file "$extension_binary" \
  --bundle-identifier io.ccswitch.agent-quota-control.widget \
  --force \
  --compile-time-extraction \
  --deployment-aware-processing

metadata_file="$extension_resources/Metadata.appintents/extract.actionsdata"
if [[ ! -f "$metadata_file" ]] || ! grep -Eq '"name"[[:space:]]*:[[:space:]]*"account"' "$metadata_file"; then
  echo "error: Widget App Intent metadata does not contain the account parameter" >&2
  exit 1
fi

xcrun swiftc \
  -swift-version 5 \
  -O \
  -target "$target_triple" \
  -sdk "$sdk_path" \
  -framework WidgetKit \
  "$widget_root/Sources/ReloadHelper/main.swift" \
  -o "$sidecar"

chmod +x "$extension_binary" "$sidecar"
plutil -lint "$extension/Contents/Info.plist" >/dev/null

echo "widget extension: $extension"
echo "reload sidecar: $sidecar"

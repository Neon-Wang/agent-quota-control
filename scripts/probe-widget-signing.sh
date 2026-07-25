#!/usr/bin/env bash

set -euo pipefail

if ! command -v xcodebuild >/dev/null 2>&1; then
  echo "error: xcodebuild is required" >&2
  exit 1
fi

if ! command -v security >/dev/null 2>&1; then
  echo "error: macOS security tool is required" >&2
  exit 1
fi

valid_identity_line="$({ security find-identity -v -p codesigning 2>/dev/null || true; } \
  | awk '/Apple Development:/ && $0 !~ /REVOKED/ { print; exit }')"

if [[ -z "$valid_identity_line" ]]; then
  echo "error: no valid Apple Development signing identity found" >&2
  exit 1
fi

identity_hash="$(awk '{ print $2 }' <<<"$valid_identity_line")"
certificate_pem="$(security find-certificate -c "Apple Development" -p 2>/dev/null || true)"
team_id="$(openssl x509 -noout -subject 2>/dev/null <<<"$certificate_pem" \
  | sed -n 's/.*OU *= *\([^,\/]*\).*/\1/p' \
  | head -n 1)"

if [[ -z "$team_id" ]]; then
  echo "error: unable to read a team ID from the development certificate" >&2
  exit 1
fi

xcode_version="$(xcodebuild -version | tr '\n' ' ')"
sdk_path="$(xcrun --sdk macosx --show-sdk-path)"

echo "xcode: $xcode_version"
echo "sdk: $sdk_path"
echo "identity: $identity_hash"
echo "team: $team_id"
echo "note: App Group provisioning and widget discovery still require bundle verification"

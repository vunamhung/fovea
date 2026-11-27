#!/usr/bin/env bash
set -euo pipefail

# Packages and signs Fovea.app with the local macOS codesigning certificate so macOS
# preserves Screen Recording, Accessibility, and Input Monitoring permissions across rebuilds.

APP_NAME="Fovea"
CERT_NAME="${FOVEA_SIGN_CERT:-VNH Local}"
BINARY_PATH="${1:-target/release/fovea}"
OUT_APP="${2:-dist/${APP_NAME}.app}"

if [[ ! -f "$BINARY_PATH" ]]; then
  echo "Binary not found at $BINARY_PATH. Run 'cargo build --release -p fovea-app' first." >&2
  exit 1
fi

pkill -f "${OUT_APP}/Contents/MacOS/fovea" 2>/dev/null || true

rm -rf "$OUT_APP"
mkdir -p "$OUT_APP/Contents/MacOS" "$OUT_APP/Contents/Resources"
cp "$BINARY_PATH" "$OUT_APP/Contents/MacOS/fovea"
chmod +x "$OUT_APP/Contents/MacOS/fovea"

cat > "$OUT_APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>fovea</string>
  <key>CFBundleIdentifier</key>
  <string>dev.fovea.studio</string>
  <key>CFBundleName</key>
  <string>Fovea</string>
  <key>CFBundleDisplayName</key>
  <string>Fovea</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>14.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSScreenCaptureUsageDescription</key>
  <string>Fovea captures your screen and window content to create polished demo videos.</string>
  <key>NSMicrophoneUsageDescription</key>
  <string>Fovea records microphone narration alongside your screen capture.</string>
  <key>NSCameraUsageDescription</key>
  <string>Fovea captures your webcam overlay for picture-in-picture demos.</string>
</dict>
</plist>
PLIST

if security find-identity -v -p codesigning | grep -q "\"$CERT_NAME\""; then
  codesign --force --deep --options runtime --sign "$CERT_NAME" "$OUT_APP" 2>/dev/null \
    || codesign --force --deep --sign "$CERT_NAME" "$OUT_APP"
  echo "Signed $OUT_APP with '$CERT_NAME'."
elif security find-identity -v -p codesigning | grep -q "\"Fovea Local Signing\""; then
  codesign --force --deep --sign "Fovea Local Signing" "$OUT_APP"
  echo "Signed $OUT_APP with 'Fovea Local Signing'."
else
  codesign --force --deep --sign - "$OUT_APP"
  echo "Signed $OUT_APP ad-hoc (no '$CERT_NAME' identity found in Keychain)."
fi

codesign --verify --deep --strict "$OUT_APP"

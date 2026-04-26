#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

CARGO="${CARGO:-$HOME/.cargo/bin/cargo}"
APP_DIR="dist/Alinea.app"
CONTENTS="$APP_DIR/Contents"
MACOS="$CONTENTS/MacOS"

"$CARGO" build --release

rm -rf "$APP_DIR"
mkdir -p "$MACOS"

cp target/release/alinea "$MACOS/Alinea"
chmod +x "$MACOS/Alinea"

cat > "$CONTENTS/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>Alinea</string>
  <key>CFBundleIdentifier</key>
  <string>dev.lgallindo.alinea</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>Alinea</string>
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
</dict>
</plist>
PLIST

echo "$APP_DIR"

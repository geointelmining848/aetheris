#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
REPO_ROOT="${SCRIPT_DIR:h}"
BIN_PATH="$REPO_ROOT/target/release/aetheris"

INSTALL_DIR="${1:-/Volumes/X10 Pro/Applications}"
APP_BUNDLE="$INSTALL_DIR/Aetheris.app"
LINK_IN_APPLICATIONS="/Applications/Aetheris.app"
CLI_LINK="$HOME/.local/bin/aetheris"
LSREG="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"

if [[ ! -x "$BIN_PATH" ]]; then
  echo "Release binary not found at $BIN_PATH — run 'cargo build --release' first." >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"

WORK_TMP="$(mktemp -d)"
trap 'rm -rf "$WORK_TMP"' EXIT

ICONSET="$WORK_TMP/Aetheris.iconset"
mkdir -p "$ICONSET"

# Render crisp 1024x1024 Aetheris icon using native macOS AppKit/CoreGraphics
cat > "$WORK_TMP/gen_icon.swift" <<'SWIFT'
import AppKit
import CoreGraphics
import Foundation

let outPath = CommandLine.arguments[1]
let size: CGFloat = 1024

let rep = NSBitmapImageRep(
    bitmapDataPlanes: nil,
    pixelsWide: Int(size),
    pixelsHigh: Int(size),
    bitsPerSample: 8,
    samplesPerPixel: 4,
    hasAlpha: true,
    isPlanar: false,
    colorSpaceName: .deviceRGB,
    bytesPerRow: 0,
    bitsPerPixel: 0
)!

NSGraphicsContext.saveGraphicsState()
let nsCtx = NSGraphicsContext(bitmapImageRep: rep)!
NSGraphicsContext.current = nsCtx
let ctx = nsCtx.cgContext

// macOS squircle-like rounded rect inset
let inset: CGFloat = 86
let rect = CGRect(x: inset, y: inset, width: size - inset * 2, height: size - inset * 2)
let radius: CGFloat = 192
let rounded = CGPath(roundedRect: rect, cornerWidth: radius, cornerHeight: radius, transform: nil)

// Subtle drop shadow
ctx.saveGState()
ctx.setShadow(offset: CGSize(width: 0, height: -18), blur: 36, color: CGColor(red: 0, green: 0, blue: 0, alpha: 0.45))
ctx.addPath(rounded)
ctx.setFillColor(CGColor(red: 0.08, green: 0.10, blue: 0.18, alpha: 1.0))
ctx.fillPath()
ctx.restoreGState()

// Deep midnight-indigo gradient fill
ctx.saveGState()
ctx.addPath(rounded)
ctx.clip()

let cs = CGColorSpaceCreateDeviceRGB()
let bgColors = [
    CGColor(red: 0.075, green: 0.090, blue: 0.155, alpha: 1.0),
    CGColor(red: 0.115, green: 0.145, blue: 0.255, alpha: 1.0),
    CGColor(red: 0.160, green: 0.195, blue: 0.345, alpha: 1.0),
] as CFArray
let bgGrad = CGGradient(colorsSpace: cs, colors: bgColors, locations: [0.0, 0.55, 1.0])!
ctx.drawLinearGradient(bgGrad, start: CGPoint(x: rect.minX, y: rect.minY), end: CGPoint(x: rect.maxX, y: rect.maxY), options: [])

// Soft radial luminous aura behind the emblem
let auraColors = [
    CGColor(red: 0.48, green: 0.68, blue: 1.0, alpha: 0.32),
    CGColor(red: 0.68, green: 0.52, blue: 0.98, alpha: 0.12),
    CGColor(red: 0.10, green: 0.13, blue: 0.24, alpha: 0.0),
] as CFArray
let auraGrad = CGGradient(colorsSpace: cs, colors: auraColors, locations: [0.0, 0.48, 1.0])!
ctx.drawRadialGradient(
    auraGrad,
    startCenter: CGPoint(x: size * 0.50, y: size * 0.58),
    startRadius: 20,
    endCenter: CGPoint(x: size * 0.50, y: size * 0.58),
    endRadius: 360,
    options: []
)

// Subtle horizontal text-line motif near lower third
let lineY: [CGFloat] = [312, 262, 212]
let lineW: [CGFloat] = [370, 440, 290]
for i in 0..<lineY.count {
    let lx = (size - lineW[i]) / 2.0
    let lr = CGRect(x: lx, y: lineY[i], width: lineW[i], height: 14)
    let lp = CGPath(roundedRect: lr, cornerWidth: 7, cornerHeight: 7, transform: nil)
    ctx.addPath(lp)
    ctx.setFillColor(CGColor(red: 0.58, green: 0.72, blue: 0.98, alpha: 0.22))
    ctx.fillPath()
}

// Crisp luminous quill / star-nib emblem in upper-center
let cx: CGFloat = size * 0.50
let cy: CGFloat = size * 0.59
let star = CGMutablePath()
star.move(to: CGPoint(x: cx, y: cy + 215))
star.addQuadCurve(to: CGPoint(x: cx + 215, y: cy), control: CGPoint(x: cx + 42, y: cy + 42))
star.addQuadCurve(to: CGPoint(x: cx, y: cy - 215), control: CGPoint(x: cx + 42, y: cy - 42))
star.addQuadCurve(to: CGPoint(x: cx - 215, y: cy), control: CGPoint(x: cx - 42, y: cy - 42))
star.addQuadCurve(to: CGPoint(x: cx, y: cy + 215), control: CGPoint(x: cx - 42, y: cy + 42))
star.closeSubpath()

ctx.saveGState()
ctx.addPath(star)
ctx.clip()
let starColors = [
    CGColor(red: 0.94, green: 0.97, blue: 1.0, alpha: 1.0),
    CGColor(red: 0.49, green: 0.81, blue: 1.0, alpha: 1.0),
    CGColor(red: 0.55, green: 0.62, blue: 0.98, alpha: 1.0),
    CGColor(red: 0.74, green: 0.60, blue: 0.98, alpha: 1.0),
] as CFArray
let starGrad = CGGradient(colorsSpace: cs, colors: starColors, locations: [0.0, 0.35, 0.72, 1.0])!
ctx.drawLinearGradient(
    starGrad,
    start: CGPoint(x: cx - 140, y: cy + 190),
    end: CGPoint(x: cx + 140, y: cy - 190),
    options: []
)
ctx.restoreGState()

// Center jewel dot
let dot = CGRect(x: cx - 20, y: cy - 20, width: 40, height: 40)
ctx.setFillColor(CGColor(red: 0.08, green: 0.11, blue: 0.21, alpha: 0.85))
ctx.fillEllipse(in: dot)

// Inner border highlight
ctx.restoreGState()
ctx.addPath(rounded)
ctx.setStrokeColor(CGColor(red: 0.56, green: 0.73, blue: 1.0, alpha: 0.35))
ctx.setLineWidth(6)
ctx.strokePath()

NSGraphicsContext.restoreGraphicsState()

let png = rep.representation(using: .png, properties: [:])!
try png.write(to: URL(fileURLWithPath: outPath))
SWIFT

/usr/bin/swift "$WORK_TMP/gen_icon.swift" "$WORK_TMP/icon_1024.png"

for sz in 16 32 128 256 512; do
  double=$((sz * 2))
  /usr/bin/sips -z "$sz" "$sz" "$WORK_TMP/icon_1024.png" --out "$ICONSET/icon_${sz}x${sz}.png" >/dev/null
  /usr/bin/sips -z "$double" "$double" "$WORK_TMP/icon_1024.png" --out "$ICONSET/icon_${sz}x${sz}@2x.png" >/dev/null
done

/usr/bin/iconutil -c icns "$ICONSET" -o "$WORK_TMP/Aetheris.icns"

# Assemble Aetheris.app atomically on X10 Pro
STAGE="$INSTALL_DIR/.Aetheris.app.stage-$$"
rm -rf "$STAGE"
mkdir -p "$STAGE/Contents/MacOS" "$STAGE/Contents/Resources"

cp "$BIN_PATH" "$STAGE/Contents/MacOS/aetheris"
chmod +x "$STAGE/Contents/MacOS/aetheris"
cp "$WORK_TMP/Aetheris.icns" "$STAGE/Contents/Resources/Aetheris.icns"

cat > "$STAGE/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>Aetheris</string>
  <key>CFBundleExecutable</key>
  <string>aetheris</string>
  <key>CFBundleIconFile</key>
  <string>Aetheris</string>
  <key>CFBundleIdentifier</key>
  <string>io.github.geointelmining848.aetheris</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>Aetheris</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.developer-tools</string>
  <key>LSMinimumSystemVersion</key>
  <string>13.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>CFBundleDocumentTypes</key>
  <array>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Text Document</string>
      <key>CFBundleTypeRole</key>
      <string>Editor</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>public.plain-text</string>
        <string>public.source-code</string>
        <string>public.text</string>
        <string>public.json</string>
        <string>public.xml</string>
        <string>public.comma-separated-values-text</string>
        <string>net.daringfireball.markdown</string>
        <string>public.data</string>
      </array>
    </dict>
  </array>
</dict>
</plist>
PLIST

/usr/bin/xattr -dr com.apple.quarantine "$STAGE" 2>/dev/null || true
/usr/bin/codesign --force --deep --sign - "$STAGE"
/usr/bin/codesign --verify --deep --strict "$STAGE"

rm -rf "$APP_BUNDLE"
mv "$STAGE" "$APP_BUNDLE"

# Remove any duplicate symlink in /Applications (macOS Apps scans both /Applications and
# LaunchServices-registered X10 Pro/Applications, so having both creates duplicate icons).
if [[ -L "$LINK_IN_APPLICATIONS" ]]; then
  "$LSREG" -u "$LINK_IN_APPLICATIONS" >/dev/null 2>&1 || true
  rm -f "$LINK_IN_APPLICATIONS"
fi

# Add convenient CLI command ~/.local/bin/aetheris
mkdir -p "${CLI_LINK:h}"
ln -sf "$APP_BUNDLE/Contents/MacOS/aetheris" "$CLI_LINK"

# Register strictly the single canonical X10 Pro app bundle with LaunchServices & Spotlight
"$LSREG" -f "$APP_BUNDLE" >/dev/null 2>&1 || true
/usr/bin/mdimport "$APP_BUNDLE" >/dev/null 2>&1 || true

echo "Installed: $APP_BUNDLE (single canonical Apps entry)"
echo "CLI linked: $CLI_LINK"

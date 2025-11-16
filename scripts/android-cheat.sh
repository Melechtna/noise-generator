#!/usr/bin/env bash
set -euo pipefail

# Usage: scripts/android-cheat.sh target/artifacts/noise-generator-debug.apk
APK_OUT="${1:-target/artifacts/noise-generator-debug.apk}"

# SDK tools
ANDROID_HOME="${ANDROID_HOME:-/opt/android-sdk}"
AAPT2="$ANDROID_HOME/build-tools/36.1/aapt2"
ZIPALIGN="$ANDROID_HOME/build-tools/36.1/zipalign"
APKSIGNER="$ANDROID_HOME/build-tools/36.1/apksigner"
PLATFORM_JAR="$ANDROID_HOME/platforms/android-30/android.jar"
DEBUG_KEY="$HOME/.android/debug.keystore"

# Inputs we own
MANIFEST="android/AndroidManifest.xml"
RES_DIR="android/res"
NATIVE_SO="target/debug/apk/lib/arm64-v8a/libnoise_generator.so"

# Build workspace
WORK="target/aapt-build"
COMPILED="$WORK/compiled"
UNALIGNED="$WORK/base.unaligned.apk"

# Sanity checks
[ -x "$AAPT2" ] || { echo "❌ aapt2 not found at $AAPT2"; exit 1; }
[ -f "$PLATFORM_JAR" ] || { echo "❌ platform jar missing: $PLATFORM_JAR"; exit 1; }
[ -f "$MANIFEST" ] || { echo "❌ Manifest missing: $MANIFEST"; exit 1; }
[ -d "$RES_DIR" ] || { echo "❌ Res dir missing: $RES_DIR"; exit 1; }
[ -f "$NATIVE_SO" ] || { echo "❌ Missing native lib: $NATIVE_SO (run 'cargo apk build --lib')"; exit 1; }

echo "🧹 Clean workspace"
rm -rf "$WORK"
mkdir -p "$WORK" "$COMPILED" "$(dirname "$APK_OUT")"

echo "🎨 aapt2 compile (res → flat)"
# compiles everything under res into /compiled/*.flat
"$AAPT2" compile --dir "$RES_DIR" -o "$COMPILED"

echo "📝 aapt2 link (manifest + compiled res → APK)"
# Produces base.unaligned.apk
"$AAPT2" link \
  -o "$UNALIGNED" \
  -I "$PLATFORM_JAR" \
  --manifest "$MANIFEST" \
  --min-sdk-version 26 \
  --target-sdk-version 30 \
  --auto-add-overlay \
  "$COMPILED"/*.flat

# Confirm it really exists (aapt2 returns 0 but still check)
[ -s "$UNALIGNED" ] || { echo "❌ aapt2 link produced no APK"; exit 1; }

echo "📌 Inject native libs (correct path + uncompressed)"
# Add the .so as stored (no compression). Use zip since aapt2 has no 'add'.
# Note: always use Unix-style path in zip entry
zip -q -0 -u "$UNALIGNED" "target/debug/apk/lib/arm64-v8a/libnoise_generator.so"
# Fix entry name to lib/arm64-v8a/... if zip recorded full path
# (zip with -j would strip dirs, but we want exact tree; this ensures correct name)
zip -q -d "$UNALIGNED" "target/debug/apk/lib/arm64-v8a/libnoise_generator.so" || true
( cd "target/debug/apk" && zip -q -0 -u "../../../$UNALIGNED" "lib/arm64-v8a/libnoise_generator.so" )

echo "🔧 zipalign"
"$ZIPALIGN" -f 4 "$UNALIGNED" "$APK_OUT"

echo "🔏 apksign (debug key)"
"$APKSIGNER" sign \
  --ks "$DEBUG_KEY" \
  --ks-pass pass:android \
  "$APK_OUT"

echo "✅ DONE → $APK_OUT"

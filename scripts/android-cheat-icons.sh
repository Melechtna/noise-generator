#!/usr/bin/env bash
set -euo pipefail

ICON="${1:-src/ui/icons/icon.png}"
OUT="android/res"

if [ ! -f "$ICON" ]; then
  echo "❌ Base icon not found: $ICON"
  exit 1
fi

# Clean + create folders we need
rm -rf "$OUT/mipmap-"* "$OUT/mipmap-anydpi-v26" "$OUT/values/ic_launcher_background.xml" 2>/dev/null || true
mkdir -p \
  "$OUT/mipmap-mdpi" \
  "$OUT/mipmap-hdpi" \
  "$OUT/mipmap-xhdpi" \
  "$OUT/mipmap-xxhdpi" \
  "$OUT/mipmap-xxxhdpi" \
  "$OUT/mipmap-anydpi-v26" \
  "$OUT/values"

# Density map for launcher icons (48dp baseline)
# mdpi 48, hdpi 72, xhdpi 96, xxhdpi 144, xxxhdpi 192
gen_set() {
  local size="$1" dir="$2"
  # main legacy icon
  magick "$ICON" -resize "${size}x${size}" "$OUT/$dir/ic_launcher.png"
  # round icon
  magick "$ICON" -resize "${size}x${size}" "$OUT/$dir/ic_launcher_round.png"
  # foreground
  magick "$ICON" -resize "${size}x${size}" "$OUT/$dir/ic_launcher_foreground.png"
}

gen_set 48  mipmap-mdpi
gen_set 72  mipmap-hdpi
gen_set 96  mipmap-xhdpi
gen_set 144 mipmap-xxhdpi
gen_set 192 mipmap-xxxhdpi

# Background as a COLOR resource (simplest; no drawable needed)
cat > "$OUT/values/ic_launcher_background.xml" <<'XML'
<resources>
    <!-- tweak to taste -->
    <color name="ic_launcher_background">#000000</color>
</resources>
XML

# Adaptive icon XMLs
cat > "$OUT/mipmap-anydpi-v26/ic_launcher.xml" <<'XML'
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background"/>
    <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
</adaptive-icon>
XML

cat > "$OUT/mipmap-anydpi-v26/ic_launcher_round.xml" <<'XML'
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background"/>
    <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
</adaptive-icon>
XML

echo "Icons generated under android/res/"

# ===== Noise Generator — Linux "drop-in" installer =====
# Expects these files next to this justfile:
#   ./noise-generator      (the executable, correct arch)
#   ./icon.png             (base app icon)
#   ./linux/
#       io.melechtna.noise-generator.desktop
#       io.melechtna.NoiseGenerator.metainfo.xml
#
# Usage:
#   just install
#   just uninstall

project := "noise-generator"
local_bin := "./{{project}}"
icon := "./icon.png"
desktop := "linux/io.melechtna.noise-generator.desktop"
metainfo := "linux/io.melechtna.NoiseGenerator.metainfo.xml"

default: install

_install_checks:
	#!/usr/bin/env bash
	set -euo pipefail

	# Detect arch to give a nicer hint if the binary is missing
	arch="$(uname -m || echo unknown)"
	case "$arch" in
	  x86_64)   suggested="noise-generator-linux-x64";;
	  aarch64)  suggested="noise-generator-linux-arm64";;
	  riscv64)  suggested="noise-generator-linux-riscv64";;
	  *)        suggested="noise-generator-linux-<your-arch>";;
	esac

	# Executable present?
	if [ ! -f "{{local_bin}}" ]; then
	  echo "❌ Missing executable: {{local_bin}}" >&2
	  echo "   Download the appropriate Linux binary (e.g. '$suggested') from your GitHub Releases," >&2
	  echo "   rename it to 'noise-generator', place it next to this justfile, and re-run:" >&2
	  echo "     mv $suggested noise-generator" >&2
	  echo "     chmod +x noise-generator" >&2
	  exit 1
	fi

	# Ensure it's executable (best effort)
	chmod +x "{{local_bin}}" || true

	# Icon & desktop metadata present?
	[ -f "{{icon}}" ]     || { echo "❌ Missing icon: {{icon}}"; exit 1; }
	[ -f "{{desktop}}" ]  || { echo "❌ Missing desktop file: {{desktop}}"; exit 1; }
	[ -f "{{metainfo}}" ] || { echo "❌ Missing metainfo: {{metainfo}}"; exit 1; }

	# ImageMagick available?
	if ! command -v magick >/dev/null 2>&1; then
	  echo "❌ ImageMagick 'magick' not found. Install it (e.g. 'sudo apt install imagemagick')." >&2
	  exit 1
	fi

install: _install_checks
	#!/usr/bin/env bash
	set -euo pipefail
	echo "▶ Installing {{project}} system-wide"
	sudo bash -euxo pipefail -c '
	  # Binary
	  install -Dm755 "{{local_bin}}" /usr/bin/{{project}}

	  # Icons (HiColor theme)
	  for s in 16 32 48 64 128 256 512; do
	    dir="/usr/share/icons/hicolor/${s}x${s}/apps"
	    mkdir -p "$dir"
	    magick "{{icon}}" -resize ${s}x${s} "$dir/io.melechtna.noise-generator.png"
	  done

	  # Desktop + AppStream metadata
	  install -Dm644 "{{desktop}}"  /usr/share/applications/io.melechtna.noise-generator.desktop
	  install -Dm644 "{{metainfo}}" /usr/share/metainfo/io.melechtna.NoiseGenerator.metainfo.xml

	  # Refresh caches (best-effort)
	  gtk-update-icon-cache -f /usr/share/icons/hicolor || true
	  update-desktop-database -q /usr/share/applications || true
	'
	echo "✅ Installed: run 'noise-generator' from your launcher or terminal"

uninstall:
	#!/usr/bin/env bash
	set -euo pipefail
	echo "▶ Uninstalling {{project}}"
	sudo bash -euxo pipefail -c '
	  rm -f /usr/bin/{{project}}
	  rm -f /usr/share/applications/io.melechtna.noise-generator.desktop
	  rm -f /usr/share/metainfo/io.melechtna.NoiseGenerator.metainfo.xml

	  for s in 16 32 48 64 128 256 512; do
	    rm -f "/usr/share/icons/hicolor/${s}x${s}/apps/io.melechtna.noise-generator.png"
	  done

	  gtk-update-icon-cache -f /usr/share/icons/hicolor || true
	  update-desktop-database -q /usr/share/applications || true
	'
	echo "✅ Uninstalled"

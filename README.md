
# Noise Generator

Cross-platform noise generator for sleep, focus, and relaxation.
Simple sliders, precise numeric fields, and per-band control to dial in white, pink, brown, blue, and anything in-between—plus an optional bass boost for that cozy brown-noise rumble.

## How it works (quick peek)

- **Audio**: uses the tinyaudio backend to stream generated noise cross platform.

- **Bands**: 3 configurable ranges; values are clamped & ordered to stay sane.

- **Seed**: random (per run) or fixed (for repeatable texture).

- **Alpha**: controls the temporal blend—closer to 1.0 = smoother, less “spitty”.

- **Bass Boost**: a gentle enhancer aimed at brown-ish profiles.

Internals are in Rust; UI is written with Slint.

## Download

Grab prebuilt binaries from **GitHub Releases**:

- **Windows**: x64 & ARM64 (MSVC)

- **MacOS**: Universal (Intel + Apple Silicon)

- **Linux**: x64, ARM64, RiscV (likely needs local compile)

- **Android**: Debug APK (you're welcome to pay for a key to sign a release build)

> iOS isn’t supported (yet). PRs/experiments welcome.

## Install (Linux)

For Linux, this repo includes a zip that provides everything except the executable:
```
.
├── justfile
├── icon.png
├── linux/
│   ├── io.melechtna.noise-generator.desktop
│   └── io.melechtna.NoiseGenerator.metainfo.xml
└── noise-generator   <-- put the downloaded Linux binary here (make it executable if nessecary)
```

Then:
```
# Ensure 'just' and ImageMagick are installed
cargo install just # or your distro package
sudo apt install imagemagick  # Debian/Ubuntu (for icon resizing)
# Install system-wide:
just install
# Uninstall:
just uninstall
```


The installer places:

- Binary → `/usr/bin/noise-generator`

- Icons → `/usr/share/icons/hicolor/*/apps/io.melechtna.noise-generator.png`

- Desktop entry → `/usr/share/applications/io.melechtna.noise-generator.desktop`

- AppStream → `/usr/share/metainfo/io.melechtna.NoiseGenerator.metainfo.xml`

**All other Systems**

The application is otherwise desinged to run stand alone, or is installed as per usual (MacOS/Android) and comes with its icon baked in. Linux needs a little extra doing for the icon to function relatively universally.

## Where is config.toml?

The app uses your OS-native config directory:

- **Linux**: `~/.config/noise-generator/config.toml`

- **MacOS**: `~/Library/Application Support/Noise Generator/config.toml`

- **Windows**: `%APPDATA%\noise-generator\config.toml`

- **Android**: `internal app storage (handled automatically)`

> If a standard path can’t be determined, the app falls back to a sane default and prints a message on first run.

## Known Issues (**Contributions welcome**)

 - **Numeric fields** (NumberField)

On all platforms there are **focus quirks** (e.g., clicking a second time may deselect; behavior differs between mouse and touch).

- **Android:** the settings screen doesn’t shift upwards when the keyboard appears.

It’s minor but visible/nioticable when using the UI; fix PRs are welcome. The android one is more, I didn't look into it. The NumberField issue is one I simply cannot work out how to resolve. As such, there's also some optimizations possible in the code as well, as I am pretty sure there's a few variables that exist in attempts to fix this that probably shouldn't be there.

Create an issue or open a PR if you can help refine either of these.

## Building from Source
**Prereqs (common)**

- **Rust** (stable) + cargo

- Internet access (to fetch crates)

**Android**

- JDK 21, Android SDK + NDK, and cargo-apk:

- cargo-apk

- aarch64-linux-android Rust cross target

- This repo also has helper scripts (`scripts/android-cheat-icons.sh`, `scripts/android-cheat.sh`) that CI uses to assemble a debug APK with proper icons. This is primarily required due to a bug in cargo-apk that doesn't allow setting up a proper name or icons, and the "hacky workaround" basically repackages the APK using aapt with a proper AndroidManifest.xml and its res/ folder which supplies thee icons.

## Contributing

PRs are welcome—especially for

    - Numeric field focus/selection polish
    - Android keyboard/inset handling
    - Small UX tweaks, translations, or accessibility improvements

Please keep changes scoped, add a brief description, and try to test on at least one desktop platform and Android, as these can be a touch flakey.
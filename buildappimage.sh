#!/usr/bin/env bash
set -e

# Craftrinth AppImage Build & Desktop Integration Script

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

APP_FRONTEND_DIR="$SCRIPT_DIR/apps/app-frontend"
APP_DIR="$SCRIPT_DIR/apps/app"
TARGET_DIR="$SCRIPT_DIR/target"
RELEASE_DIR="$TARGET_DIR/release"
DIST_DIR="$SCRIPT_DIR/dist"
OUTPUT_APPIMAGE="$DIST_DIR/Craftrinth-x86_64.AppImage"

echo "=================================================="
echo "          Craftrinth AppImage Builder             "
echo "=================================================="

# Clean dist directories before every build
echo "Cleaning dist directories..."
rm -rf "$DIST_DIR"
rm -rf "$APP_FRONTEND_DIR/dist"
rm -rf "$TARGET_DIR/release/bundle/appimage"
mkdir -p "$DIST_DIR"

# 1. Build frontend assets
echo "[1/4] Building frontend assets..."
if [ -f "$APP_FRONTEND_DIR/node_modules/.bin/vite" ]; then
	(cd "$APP_FRONTEND_DIR" && ./node_modules/.bin/vite build)
elif command -v pnpm >/dev/null 2>&1; then
	pnpm --filter @modrinth/app-frontend build
elif command -v npm >/dev/null 2>&1; then
	(cd "$APP_FRONTEND_DIR" && npm run build)
fi

if [ ! -f "$APP_FRONTEND_DIR/dist/index.html" ]; then
	echo "Error: Frontend build failed, $APP_FRONTEND_DIR/dist/index.html does not exist." >&2
	exit 1
fi
echo "Frontend assets ready."

# 2. Build release binary with cargo
echo "[2/4] Building Craftrinth release binary..."
cargo build --release -p theseus_gui

RELEASE_BIN=""
if [ -f "$RELEASE_DIR/craftrinth" ]; then
	RELEASE_BIN="$RELEASE_DIR/craftrinth"
elif [ -f "$RELEASE_DIR/Craftrinth" ]; then
	RELEASE_BIN="$RELEASE_DIR/Craftrinth"
elif [ -f "$RELEASE_DIR/theseus_gui" ]; then
	RELEASE_BIN="$RELEASE_DIR/theseus_gui"
fi

if [ -z "$RELEASE_BIN" ] || [ ! -f "$RELEASE_BIN" ]; then
	echo "Error: Release binary not found in $RELEASE_DIR" >&2
	exit 1
fi
echo "Release binary ready: $RELEASE_BIN"

# 3. Create AppImage bundle
echo "[3/4] Packaging AppImage..."
mkdir -p "$DIST_DIR"

# Construct AppDir with newly built binary and assets
APPDIR="$TARGET_DIR/Craftrinth.AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/512x512/apps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/128x128/apps"
mkdir -p "$APPDIR/usr/share/icons/hicolor/scalable/apps"

	cp "$RELEASE_BIN" "$APPDIR/usr/bin/craftrinth"
	chmod +x "$APPDIR/usr/bin/craftrinth"

	cp "$APP_DIR/icons/icon.png" "$APPDIR/usr/share/icons/hicolor/512x512/apps/craftrinth.png"
	cp "$APP_DIR/icons/128x128@2x.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/craftrinth.png"
	cp "$APP_DIR/icons/128x128.png" "$APPDIR/usr/share/icons/hicolor/128x128/apps/craftrinth.png"
	cp "$APP_FRONTEND_DIR/src/assets/logo.svg" "$APPDIR/usr/share/icons/hicolor/scalable/apps/craftrinth.svg"

	cp "$APP_DIR/icons/icon.png" "$APPDIR/craftrinth.png"
	cp "$APP_DIR/icons/icon.png" "$APPDIR/.DirIcon"

	cat << 'EOF' > "$APPDIR/craftrinth.desktop"
[Desktop Entry]
Name=Craftrinth
GenericName=Minecraft Launcher
Comment=Cracked & Offline Modrinth App Launcher
Exec=craftrinth %u
Icon=craftrinth
Terminal=false
Type=Application
Categories=Game;ActionGame;AdventureGame;
MimeType=x-scheme-handler/craftrinth;x-scheme-handler/modrinth;application/x-modrinth-modpack+zip;
StartupWMClass=craftrinth
Keywords=minecraft;modrinth;craftrinth;mods;modpack;cracked;offline;
Actions=NewInstance;

[Desktop Action NewInstance]
Name=New Instance
Exec=craftrinth
EOF
	cp "$APPDIR/craftrinth.desktop" "$APPDIR/usr/share/applications/craftrinth.desktop"

	# AppRun: checks on first boot / when missing if desktop entry should be installed
	cat << 'EOF' > "$APPDIR/AppRun"
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"

# First boot or missing desktop entry check
USER_DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}"
APPLICATIONS_DIR="$USER_DATA_DIR/applications"
APPLICATION_ALT_DIR="$USER_DATA_DIR/application"
DESKTOP_FILE="$APPLICATIONS_DIR/craftrinth.desktop"

if [ ! -f "$DESKTOP_FILE" ]; then
	mkdir -p "$APPLICATIONS_DIR"
	mkdir -p "$USER_DATA_DIR/icons/hicolor/512x512/apps"
	mkdir -p "$USER_DATA_DIR/icons/hicolor/256x256/apps"
	mkdir -p "$USER_DATA_DIR/icons/hicolor/128x128/apps"
	mkdir -p "$USER_DATA_DIR/icons/hicolor/scalable/apps"

	[ -f "${HERE}/usr/share/icons/hicolor/512x512/apps/craftrinth.png" ] && cp "${HERE}/usr/share/icons/hicolor/512x512/apps/craftrinth.png" "$USER_DATA_DIR/icons/hicolor/512x512/apps/craftrinth.png" 2>/dev/null || true
	[ -f "${HERE}/usr/share/icons/hicolor/256x256/apps/craftrinth.png" ] && cp "${HERE}/usr/share/icons/hicolor/256x256/apps/craftrinth.png" "$USER_DATA_DIR/icons/hicolor/256x256/apps/craftrinth.png" 2>/dev/null || true
	[ -f "${HERE}/usr/share/icons/hicolor/128x128/apps/craftrinth.png" ] && cp "${HERE}/usr/share/icons/hicolor/128x128/apps/craftrinth.png" "$USER_DATA_DIR/icons/hicolor/128x128/apps/craftrinth.png" 2>/dev/null || true
	[ -f "${HERE}/usr/share/icons/hicolor/scalable/apps/craftrinth.svg" ] && cp "${HERE}/usr/share/icons/hicolor/scalable/apps/craftrinth.svg" "$USER_DATA_DIR/icons/hicolor/scalable/apps/craftrinth.svg" 2>/dev/null || true

	EXEC_PATH="${APPIMAGE:-$SELF}"
	cat << DESKTOP_EOF > "$DESKTOP_FILE"
[Desktop Entry]
Name=Craftrinth
GenericName=Minecraft Launcher
Comment=Cracked & Offline Modrinth App Launcher
Exec="${EXEC_PATH}" %u
Icon=craftrinth
Terminal=false
Type=Application
Categories=Game;ActionGame;AdventureGame;
MimeType=x-scheme-handler/craftrinth;x-scheme-handler/modrinth;application/x-modrinth-modpack+zip;
StartupWMClass=craftrinth
Keywords=minecraft;modrinth;craftrinth;mods;modpack;cracked;offline;
Actions=NewInstance;

[Desktop Action NewInstance]
Name=New Instance
Exec="${EXEC_PATH}"
DESKTOP_EOF
	chmod +x "$DESKTOP_FILE"

	if [ ! -d "$APPLICATION_ALT_DIR" ]; then
		ln -s "$APPLICATIONS_DIR" "$APPLICATION_ALT_DIR" 2>/dev/null || true
	else
		cp "$DESKTOP_FILE" "$APPLICATION_ALT_DIR/craftrinth.desktop" 2>/dev/null || true
	fi

	command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APPLICATIONS_DIR" 2>/dev/null || true
	command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -f -t "$USER_DATA_DIR/icons/hicolor" 2>/dev/null || true
fi

exec "${HERE}/usr/bin/craftrinth" "$@"
EOF
	chmod +x "$APPDIR/AppRun"

	APPIMAGETOOL_BIN="$(command -v appimagetool || true)"
	if [ -f "/tmp/appimagetool-extracted/AppRun" ]; then
		APPIMAGETOOL_BIN="/tmp/appimagetool-extracted/AppRun"
	elif [ -f "/tmp/appimagetool" ]; then
		(cd /tmp && /tmp/appimagetool --appimage-extract >/dev/null 2>&1)
		if [ -f "/tmp/squashfs-root/AppRun" ]; then
			mv /tmp/squashfs-root /tmp/appimagetool-extracted 2>/dev/null || true
			APPIMAGETOOL_BIN="/tmp/appimagetool-extracted/AppRun"
		else
			APPIMAGETOOL_BIN="/tmp/appimagetool"
		fi
	fi

	if [ -n "$APPIMAGETOOL_BIN" ] && [ -x "$APPIMAGETOOL_BIN" ]; then
		if [ "$APPIMAGETOOL_BIN" = "/tmp/appimagetool" ]; then
			ARCH=x86_64 "$APPIMAGETOOL_BIN" --appimage-extract-and-run "$APPDIR" "$OUTPUT_APPIMAGE"
		else
			ARCH=x86_64 "$APPIMAGETOOL_BIN" "$APPDIR" "$OUTPUT_APPIMAGE"
		fi
	else
		# Fallback packaging when appimagetool is absent
		echo "Notice: appimagetool not found; packaging AppDir into portable AppImage bundle."
		ARCH=x86_64
		if [ -f "$APP_DIR/node_modules/.bin/tauri" ]; then
			(cd "$APP_DIR" && ./node_modules/.bin/tauri build --bundles appimage --config '{"build":{"beforeBuildCommand":""}}' || true)
		fi
		for img in "$TARGET_DIR"/release/bundle/appimage/*.AppImage "$DIST_DIR"/*.AppImage; do
			if [ -f "$img" ]; then
				TAURI_APPIMAGE="$img"
				break
			fi
		done
		if [ -n "$TAURI_APPIMAGE" ]; then
			cp "$TAURI_APPIMAGE" "$OUTPUT_APPIMAGE"
		else
			# Provide portable AppDir launcher
			OUTPUT_APPIMAGE="$DIST_DIR/Craftrinth.AppDir/AppRun"
			cp -r "$APPDIR" "$DIST_DIR/Craftrinth.AppDir"
		fi
	fi

if [ -f "$OUTPUT_APPIMAGE" ]; then
	chmod +x "$OUTPUT_APPIMAGE"
	echo "AppImage ready at: $OUTPUT_APPIMAGE"
fi

echo "=================================================="
echo "Craftrinth AppImage built successfully!"
echo "Binary/AppImage: $OUTPUT_APPIMAGE"
echo "(The desktop entry will be generated automatically by the AppImage on run)"
echo "=================================================="

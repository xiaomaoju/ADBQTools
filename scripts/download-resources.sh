#!/usr/bin/env bash
# Download embedded resources (adb, bundletool, JRE) for CI builds.
# Usage: ./scripts/download-resources.sh [macos|windows]
set -euo pipefail

PLATFORM="${1:-}"
if [[ -z "$PLATFORM" ]]; then
  case "$(uname -s)" in
    Darwin) PLATFORM="macos" ;;
    *)      PLATFORM="windows" ;;
  esac
fi

RESOURCE_DIR="src-tauri/resources"

# --- Versions ---
BUNDLETOOL_VERSION="1.17.2"
JRE_VERSION="21.0.3+9"
JRE_VERSION_URL="21.0.3%2B9"  # URL-encoded

echo "=== Downloading resources for $PLATFORM ==="

# --- bundletool (shared) ---
SHARED_DIR="$RESOURCE_DIR/shared"
mkdir -p "$SHARED_DIR"
if [[ ! -f "$SHARED_DIR/bundletool.jar" ]]; then
  echo "[1/3] Downloading bundletool $BUNDLETOOL_VERSION..."
  curl -fSL -o "$SHARED_DIR/bundletool.jar" \
    "https://github.com/google/bundletool/releases/download/$BUNDLETOOL_VERSION/bundletool-all-$BUNDLETOOL_VERSION.jar"
else
  echo "[1/3] bundletool already exists, skipping."
fi

# --- adb (platform-specific) ---
if [[ "$PLATFORM" == "macos" ]]; then
  TARGET_DIR="$RESOURCE_DIR/macos"
  mkdir -p "$TARGET_DIR"
  if [[ ! -f "$TARGET_DIR/adb" ]]; then
    echo "[2/3] Downloading Android platform-tools (macOS)..."
    curl -fSL -o /tmp/platform-tools.zip \
      "https://dl.google.com/android/repository/platform-tools-latest-darwin.zip"
    unzip -q -o /tmp/platform-tools.zip -d /tmp/pt
    cp /tmp/pt/platform-tools/adb "$TARGET_DIR/adb"
    chmod +x "$TARGET_DIR/adb"
    rm -rf /tmp/platform-tools.zip /tmp/pt
  else
    echo "[2/3] adb already exists, skipping."
  fi
elif [[ "$PLATFORM" == "windows" ]]; then
  TARGET_DIR="$RESOURCE_DIR/windows"
  mkdir -p "$TARGET_DIR"
  if [[ ! -f "$TARGET_DIR/adb.exe" ]]; then
    echo "[2/3] Downloading Android platform-tools (Windows)..."
    curl -fSL -o /tmp/platform-tools.zip \
      "https://dl.google.com/android/repository/platform-tools-latest-windows.zip"
    unzip -q -o /tmp/platform-tools.zip -d /tmp/pt
    cp /tmp/pt/platform-tools/adb.exe "$TARGET_DIR/adb.exe"
    cp /tmp/pt/platform-tools/AdbWinApi.dll "$TARGET_DIR/AdbWinApi.dll"
    cp /tmp/pt/platform-tools/AdbWinUsbApi.dll "$TARGET_DIR/AdbWinUsbApi.dll"
    rm -rf /tmp/platform-tools.zip /tmp/pt
  else
    echo "[2/3] adb.exe already exists, skipping."
  fi
fi

# --- JRE (Eclipse Temurin, platform-specific) ---
# JRE is packaged as jre.zip and extracted at runtime to avoid Tauri glob issues
if [[ "$PLATFORM" == "macos" ]]; then
  JRE_ZIP="$RESOURCE_DIR/macos/jre.zip"
  if [[ ! -f "$JRE_ZIP" ]]; then
    echo "[3/3] Downloading JRE $JRE_VERSION (macOS aarch64)..."
    curl -fSL -o /tmp/jre.tar.gz \
      "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-${JRE_VERSION_URL}/OpenJDK21U-jre_aarch64_mac_hotspot_${JRE_VERSION/+/_}.tar.gz"
    mkdir -p /tmp/jre_stage
    tar xzf /tmp/jre.tar.gz -C /tmp
    # Temurin extracts to jdk-xxx-jre/ — we want the Contents dir
    mkdir -p /tmp/jre_stage/jre
    cp -R /tmp/jdk-*/Contents /tmp/jre_stage/jre/
    chmod +x /tmp/jre_stage/jre/Contents/Home/bin/java /tmp/jre_stage/jre/Contents/Home/bin/keytool
    # Create zip with jre/ as root directory
    (cd /tmp/jre_stage && zip -qr "$OLDPWD/$JRE_ZIP" jre/)
    rm -rf /tmp/jre.tar.gz /tmp/jdk-* /tmp/jre_stage
  else
    echo "[3/3] JRE zip already exists, skipping."
  fi
elif [[ "$PLATFORM" == "windows" ]]; then
  JRE_ZIP="$RESOURCE_DIR/windows/jre.zip"
  if [[ ! -f "$JRE_ZIP" ]]; then
    echo "[3/3] Downloading JRE $JRE_VERSION (Windows x64)..."
    curl -fSL -o /tmp/jre.zip \
      "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-${JRE_VERSION_URL}/OpenJDK21U-jre_x64_windows_hotspot_${JRE_VERSION/+/_}.zip"
    unzip -q -o /tmp/jre.zip -d /tmp
    # Temurin extracts to jdk-xxx-jre/ — repackage with jre/ as root
    mkdir -p /tmp/jre_stage
    mv /tmp/jdk-*/ /tmp/jre_stage/jre
    # Use 7z on Windows (no zip command), fall back to zip on other platforms
    if command -v 7z &>/dev/null; then
      (cd /tmp/jre_stage && 7z a -tzip -bso0 -bsp0 "$OLDPWD/$JRE_ZIP" jre/)
    else
      (cd /tmp/jre_stage && zip -qr "$OLDPWD/$JRE_ZIP" jre/)
    fi
    rm -rf /tmp/jre.zip /tmp/jre_stage
  else
    echo "[3/3] JRE zip already exists, skipping."
  fi
fi

echo "=== Done! Resources ready in $RESOURCE_DIR ==="

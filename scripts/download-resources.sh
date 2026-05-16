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
PLATFORM_TOOLS_VERSION="35.0.2"
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
      "https://dl.google.com/android/repository/platform-tools_r${PLATFORM_TOOLS_VERSION}-darwin.zip"
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
      "https://dl.google.com/android/repository/platform-tools_r${PLATFORM_TOOLS_VERSION}-windows.zip"
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
if [[ "$PLATFORM" == "macos" ]]; then
  JRE_DIR="$RESOURCE_DIR/macos/jre"
  if [[ ! -d "$JRE_DIR" ]]; then
    echo "[3/3] Downloading JRE $JRE_VERSION (macOS aarch64)..."
    curl -fSL -o /tmp/jre.tar.gz \
      "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-${JRE_VERSION_URL}/OpenJDK21U-jre_aarch64_mac_hotspot_${JRE_VERSION/+/_}.tar.gz"
    mkdir -p "$JRE_DIR"
    tar xzf /tmp/jre.tar.gz -C /tmp
    # Temurin extracts to jdk-xxx-jre/ — copy the Contents dir
    cp -R /tmp/jdk-*/Contents "$JRE_DIR/"
    chmod +x "$JRE_DIR/Contents/Home/bin/java" "$JRE_DIR/Contents/Home/bin/keytool"
    rm -rf /tmp/jre.tar.gz /tmp/jdk-*
  else
    echo "[3/3] JRE already exists, skipping."
  fi
elif [[ "$PLATFORM" == "windows" ]]; then
  JRE_DIR="$RESOURCE_DIR/windows/jre"
  if [[ ! -d "$JRE_DIR" ]]; then
    echo "[3/3] Downloading JRE $JRE_VERSION (Windows x64)..."
    curl -fSL -o /tmp/jre.zip \
      "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-${JRE_VERSION_URL}/OpenJDK21U-jre_x64_windows_hotspot_${JRE_VERSION/+/_}.zip"
    unzip -q -o /tmp/jre.zip -d /tmp
    # Temurin extracts to jdk-xxx-jre/ — move contents into jre/
    mkdir -p "$JRE_DIR"
    cp -R /tmp/jdk-*/* "$JRE_DIR/"
    rm -rf /tmp/jre.zip /tmp/jdk-*
  else
    echo "[3/3] JRE already exists, skipping."
  fi
fi

echo "=== Done! Resources ready in $RESOURCE_DIR ==="

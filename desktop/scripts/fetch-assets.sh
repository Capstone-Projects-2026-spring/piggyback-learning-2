#!/usr/bin/env bash

set -euo pipefail

# Always resolve paths relative to the repo root, not cwd
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TAURI_DIR="$REPO_ROOT/src-tauri"

ARCH=$(uname -m)
OS=$(uname -s)
LIBVOSK_VER="0.3.42"
YT_DLP_VER=$(curl -s https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest \
  | grep '"tag_name"' | cut -d'"' -f4)

mkdir -p "$TAURI_DIR/binaries"
mkdir -p "$TAURI_DIR/models/vosk-spk-model"
mkdir -p "$TAURI_DIR/models/vosk-lgraph-model"
mkdir -p "$TAURI_DIR/models/mood"
mkdir -p "$TAURI_DIR/lib"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
check_file() { [[ -f "$1" && -s "$1" ]]; }
check_dir()  { [[ -d "$1" && -n "$(ls -A "$1" 2>/dev/null)" ]]; }

# ---------------------------------------------------------------------------
# Normalize arch / triple
# ---------------------------------------------------------------------------
case "$ARCH" in
  x86_64)        TAURI_ARCH="x86_64" ;;
  aarch64|arm64) TAURI_ARCH="aarch64" ;;
  *)             echo "Unsupported arch: $ARCH"; exit 1 ;;
esac

case "$OS" in
  Linux)  TAURI_OS="unknown-linux-gnu" ;;
  Darwin) TAURI_OS="apple-darwin" ;;
  *)      echo "Unsupported OS — use fetch-assets.ps1 on Windows"; exit 1 ;;
esac

TRIPLE="${TAURI_ARCH}-${TAURI_OS}"
echo "==> Target triple: $TRIPLE"
echo "==> Repo root: $REPO_ROOT"

# ---------------------------------------------------------------------------
# yt-dlp
# ---------------------------------------------------------------------------
YT_DLP_DEST="$TAURI_DIR/binaries/yt-dlp-${TRIPLE}"
if check_file "$YT_DLP_DEST"; then
  echo "==> [skip] yt-dlp already present"
else
  echo "==> Fetching yt-dlp ($YT_DLP_VER)..."
  if [ "$OS" = "Linux" ]; then
    curl -fSL "https://github.com/yt-dlp/yt-dlp/releases/download/${YT_DLP_VER}/yt-dlp_linux" \
      -o "$YT_DLP_DEST"
  elif [ "$OS" = "Darwin" ]; then
    curl -fSL "https://github.com/yt-dlp/yt-dlp/releases/download/${YT_DLP_VER}/yt-dlp_macos" \
      -o "$YT_DLP_DEST"
  fi
  chmod +x "$YT_DLP_DEST"
  echo "   Done."
fi

# ---------------------------------------------------------------------------
# ffmpeg
# ---------------------------------------------------------------------------
FFMPEG_DEST="$TAURI_DIR/binaries/ffmpeg-${TRIPLE}"
if check_file "$FFMPEG_DEST"; then
  echo "==> [skip] ffmpeg already present"
else
  echo "==> Fetching ffmpeg..."
  if [ "$OS" = "Linux" ]; then
    curl -fSL "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz" \
      | tar -xJ --strip-components=1 -C /tmp --wildcards "*/ffmpeg"
    mv /tmp/ffmpeg "$FFMPEG_DEST"
  elif [ "$OS" = "Darwin" ]; then
    curl -fSL "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip" -o /tmp/ffmpeg.zip
    unzip -o /tmp/ffmpeg.zip -d /tmp/ffmpeg-mac
    mv /tmp/ffmpeg-mac/ffmpeg "$FFMPEG_DEST"
    rm -rf /tmp/ffmpeg.zip /tmp/ffmpeg-mac
  fi
  chmod +x "$FFMPEG_DEST"
  echo "   Done."
fi

# ---------------------------------------------------------------------------
# Vosk speaker model
# ---------------------------------------------------------------------------
if check_dir "$TAURI_DIR/models/vosk-spk-model"; then
  echo "==> [skip] Vosk speaker model already present"
else
  echo "==> Fetching Vosk speaker model..."
  curl -fSL "https://alphacephei.com/vosk/models/vosk-model-spk-0.4.zip" \
    -o /tmp/vosk-spk.zip
  unzip -o /tmp/vosk-spk.zip -d /tmp/vosk-spk-extracted
  EXTRACTED=$(find /tmp/vosk-spk-extracted -maxdepth 1 -mindepth 1 -type d | head -1)
  cp -r "$EXTRACTED"/. "$TAURI_DIR/models/vosk-spk-model/"
  rm -rf /tmp/vosk-spk.zip /tmp/vosk-spk-extracted
  echo "   Done."
fi

# ---------------------------------------------------------------------------
# Vosk lgraph model
# ---------------------------------------------------------------------------
if check_dir "$TAURI_DIR/models/vosk-lgraph-model"; then
  echo "==> [skip] Vosk lgraph model already present"
else
  echo "==> Fetching Vosk lgraph model..."
  curl -fSL "https://alphacephei.com/vosk/models/vosk-model-en-us-0.22-lgraph.zip" \
    -o /tmp/vosk-lgraph.zip
  unzip -o /tmp/vosk-lgraph.zip -d /tmp/vosk-lgraph-extracted
  EXTRACTED=$(find /tmp/vosk-lgraph-extracted -maxdepth 1 -mindepth 1 -type d | head -1)
  cp -r "$EXTRACTED"/. "$TAURI_DIR/models/vosk-lgraph-model/"
  rm -rf /tmp/vosk-lgraph.zip /tmp/vosk-lgraph-extracted
  echo "   Done."
fi

# ---------------------------------------------------------------------------
# libvosk — download ALL platforms so tauri.conf.json stays static
# ---------------------------------------------------------------------------
echo "==> Fetching libvosk ${LIBVOSK_VER} (all platforms)..."

# Linux
if check_file "$TAURI_DIR/lib/libvosk.so"; then
  echo "   [skip] libvosk.so already present"
else
  curl -fSL "https://github.com/alphacep/vosk-api/releases/download/v${LIBVOSK_VER}/vosk-linux-x86_64-${LIBVOSK_VER}.zip" \
    -o /tmp/libvosk-linux.zip
  unzip -o /tmp/libvosk-linux.zip -d /tmp/libvosk-linux
  VDIR=$(find /tmp/libvosk-linux -name "libvosk.so" | head -1 | xargs dirname)
  cp "$VDIR/libvosk.so"  "$TAURI_DIR/lib/libvosk.so"
  cp "$VDIR/vosk_api.h"  "$TAURI_DIR/lib/vosk_api.h"
  rm -rf /tmp/libvosk-linux.zip /tmp/libvosk-linux
  echo "   libvosk.so done."
fi

# macOS
if check_file "$TAURI_DIR/lib/libvosk.dylib"; then
  echo "   [skip] libvosk.dylib already present"
else
  curl -fSL "https://github.com/alphacep/vosk-api/releases/download/v${LIBVOSK_VER}/vosk-osx-${LIBVOSK_VER}.zip" \
    -o /tmp/libvosk-osx.zip
  unzip -o /tmp/libvosk-osx.zip -d /tmp/libvosk-osx
  VDIR=$(find /tmp/libvosk-osx -name "libvosk.dylib" | head -1 | xargs dirname)
  cp "$VDIR/libvosk.dylib" "$TAURI_DIR/lib/libvosk.dylib"
  # vosk_api.h is identical across platforms, only copy if not already there
  if ! check_file "$TAURI_DIR/lib/vosk_api.h"; then
    cp "$VDIR/vosk_api.h" "$TAURI_DIR/lib/vosk_api.h"
  fi
  rm -rf /tmp/libvosk-osx.zip /tmp/libvosk-osx
  echo "   libvosk.dylib done."
fi

# Windows
if check_file "$TAURI_DIR/lib/libvosk.dll"; then
  echo "   [skip] libvosk.dll already present"
else
  curl -fSL "https://github.com/alphacep/vosk-api/releases/download/v${LIBVOSK_VER}/vosk-win64-${LIBVOSK_VER}.zip" \
    -o /tmp/libvosk-win.zip
  unzip -o /tmp/libvosk-win.zip -d /tmp/libvosk-win
  VDIR=$(find /tmp/libvosk-win -name "libvosk.dll" | head -1 | xargs dirname)
  cp "$VDIR/libvosk.dll" "$TAURI_DIR/lib/libvosk.dll"
  rm -rf /tmp/libvosk-win.zip /tmp/libvosk-win
  echo "   libvosk.dll done."
fi

# ---------------------------------------------------------------------------
# ONNX mood model
# ---------------------------------------------------------------------------
if check_file "$TAURI_DIR/models/mood/emotion-ferplus-8.onnx"; then
  echo "==> [skip] ONNX mood model already present"
else
  echo "==> Fetching ONNX mood model..."
  curl -fSL "https://github.com/onnx/models/raw/main/validated/vision/body_analysis/emotion_ferplus/model/emotion-ferplus-8.onnx" \
    -o "$TAURI_DIR/models/mood/emotion-ferplus-8.onnx"
  echo "   Done."
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "All assets ready. Bundle layout:"
find "$TAURI_DIR/binaries" "$TAURI_DIR/models" "$TAURI_DIR/lib" -type f | sort

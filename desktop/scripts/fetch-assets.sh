#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODEL_DIR="$REPO_ROOT/src-tauri/models"
BIN_DIR="$REPO_ROOT/src-tauri/binaries"

mkdir -p "$MODEL_DIR"
mkdir -p "$BIN_DIR"

download_if_missing() {
    local path="$1"
    local url="$2"
    local label="$3"
    if [ -f "$path" ]; then
        echo "$label already present, skipping"
    else
        echo "Downloading $label..."
        curl -L "$url" -o "$path"
        echo "  -> $path"
    fi
}

# ── Detect platform ───────────────────────────────────────────────────────────
case "$(uname -s)" in
    Linux*)
        TRIPLE="x86_64-unknown-linux-gnu"
        YTDLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
        FFMPEG_URL="https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-linux-x64"
        BIN_EXT=""
        ;;
    Darwin*)
        TRIPLE="x86_64-apple-darwin"
        YTDLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
        FFMPEG_URL="https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-darwin-x64"
        BIN_EXT=""
        ;;
    *)
        echo "Unsupported OS: $(uname -s)"
        exit 1
        ;;
esac

# ── Whisper STT model ─────────────────────────────────────────────────────────
download_if_missing \
    "$MODEL_DIR/ggml-base.en.bin" \
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" \
    "whisper base.en (~148MB)"

# ── WeSpeaker ONNX ────────────────────────────────────────────────────────────
download_if_missing \
    "$MODEL_DIR/wespeaker.onnx" \
    "https://huggingface.co/Wespeaker/wespeaker-voxceleb-resnet34-LM/resolve/main/voxceleb_resnet34_LM.onnx" \
    "wespeaker ONNX (~25MB)"

# ── Emotion ONNX ──────────────────────────────────────────────────────────────
download_if_missing \
    "$MODEL_DIR/emotion-ferplus-8.onnx" \
    "https://github.com/onnx/models/raw/main/validated/vision/body_analysis/emotion_ferplus/model/emotion-ferplus-8.onnx" \
    "emotion ONNX"

# ── Ultraface ONNX ────────────────────────────────────────────────────────────
download_if_missing \
    "$MODEL_DIR/ultraface.onnx" \
    "https://github.com/Linzaer/Ultra-Light-Fast-Generic-Face-Detector-1MB/raw/master/models/onnx/version-RFB-320.onnx" \
    "ultraface ONNX (~1MB)"

# ── yt-dlp ────────────────────────────────────────────────────────────────────
YTDLP_BIN="$BIN_DIR/yt-dlp-$TRIPLE$BIN_EXT"
download_if_missing "$YTDLP_BIN" "$YTDLP_URL" "yt-dlp ($TRIPLE)"
chmod +x "$YTDLP_BIN"

# ── ffmpeg ────────────────────────────────────────────────────────────────────
FFMPEG_BIN="$BIN_DIR/ffmpeg-$TRIPLE$BIN_EXT"
download_if_missing "$FFMPEG_BIN" "$FFMPEG_URL" "ffmpeg ($TRIPLE)"
chmod +x "$FFMPEG_BIN"

echo ""
echo "All assets ready."

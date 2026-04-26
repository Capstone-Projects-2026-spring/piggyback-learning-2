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
        PIPER_URL="https://github.com/rhasspy/piper/releases/latest/download/piper_linux_x86_64.tar.gz"
        BIN_EXT=""
        ;;
    Darwin*)
        TRIPLE="x86_64-apple-darwin"
        YTDLP_URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
        FFMPEG_URL="https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-darwin-x64"
        PIPER_URL="https://github.com/rhasspy/piper/releases/latest/download/piper_macos_x64.tar.gz"
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

# ── Piper Alba voice model (all platforms) ────────────────────────────────────
download_if_missing \
    "$MODEL_DIR/en_GB-alba-medium.onnx" \
    "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/en_GB-alba-medium.onnx" \
    "Piper Alba voice (~60MB)"
download_if_missing \
    "$MODEL_DIR/en_GB-alba-medium.onnx.json" \
    "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/en_GB-alba-medium.onnx.json" \
    "Piper Alba config"

# ── Piper binary ──────────────────────────────────────────────────────────────
PIPER_BIN="$BIN_DIR/piper-tts-$TRIPLE$BIN_EXT"
if [ -f "$PIPER_BIN" ]; then
    echo "piper-tts already present, skipping"
else
    case "$(uname -s)" in
        Linux*)
            if command -v piper-tts &>/dev/null; then
                # Already installed via package manager (e.g. AUR piper-tts-bin)
                cp "$(command -v piper-tts)" "$PIPER_BIN"
                chmod +x "$PIPER_BIN"
                echo "  -> $PIPER_BIN (copied from $(command -v piper-tts))"
            else
                echo "Downloading piper binary..."
                cd /tmp
                curl -L "$PIPER_URL" -o piper.tar.gz
                tar -xzf piper.tar.gz
                cp piper/piper "$PIPER_BIN"
                chmod +x "$PIPER_BIN"
                rm -rf piper piper.tar.gz
                echo "  -> $PIPER_BIN"
            fi
            ;;
        Darwin*)
            if command -v piper-tts &>/dev/null; then
                cp "$(command -v piper-tts)" "$PIPER_BIN"
                chmod +x "$PIPER_BIN"
                echo "  -> $PIPER_BIN (copied from $(command -v piper-tts))"
            else
                echo "Downloading piper binary..."
                cd /tmp
                curl -L "$PIPER_URL" -o piper.tar.gz
                tar -xzf piper.tar.gz
                cp piper/piper "$PIPER_BIN"
                chmod +x "$PIPER_BIN"
                rm -rf piper piper.tar.gz
                echo "  -> $PIPER_BIN"
            fi
            ;;
    esac
fi

# ── TTS system dependencies (Linux only) ─────────────────────────────────────
case "$(uname -s)" in
    Linux*)
        echo "Installing Linux audio dependency for Piper..."
        if command -v pacman &>/dev/null; then
            sudo pacman -S --noconfirm alsa-utils
        elif command -v apt-get &>/dev/null; then
            sudo apt-get install -y alsa-utils
        elif command -v dnf &>/dev/null; then
            sudo dnf install -y alsa-utils
        elif command -v zypper &>/dev/null; then
            sudo zypper install -y alsa-utils
        fi
        ;;
esac

# ── yt-dlp ────────────────────────────────────────────────────────────────────
YTDLP_BIN="$BIN_DIR/yt-dlp-$TRIPLE$BIN_EXT"
download_if_missing "$YTDLP_BIN" "$YTDLP_URL" "yt-dlp ($TRIPLE)"
chmod +x "$YTDLP_BIN"

# ── ffmpeg ────────────────────────────────────────────────────────────────────
FFMPEG_BIN="$BIN_DIR/ffmpeg-$TRIPLE$BIN_EXT"
download_if_missing "$FFMPEG_BIN" "$FFMPEG_URL" "ffmpeg ($TRIPLE)"
chmod +x "$FFMPEG_BIN"

# ── mpv ───────────────────────────────────────────────────────────────────────
MPV_BIN="$BIN_DIR/mpv-$TRIPLE$BIN_EXT"
if [ -f "$MPV_BIN" ]; then
    echo "mpv already present, skipping"
else
    if ! command -v mpv &>/dev/null; then
        echo "Installing mpv..."
        case "$(uname -s)" in
            Linux*)
                if command -v pacman &>/dev/null; then
                    sudo pacman -S --noconfirm mpv
                elif command -v apt-get &>/dev/null; then
                    sudo apt-get install -y mpv
                elif command -v dnf &>/dev/null; then
                    sudo dnf install -y mpv
                elif command -v zypper &>/dev/null; then
                    sudo zypper install -y mpv
                elif command -v flatpak &>/dev/null; then
                    flatpak install -y flathub io.mpv.Mpv
                else
                    echo "ERROR: Could not detect package manager. Install mpv manually."
                    exit 1
                fi
                ;;
            Darwin*)
                if command -v brew &>/dev/null; then
                    brew install mpv
                else
                    echo "ERROR: Homebrew not found. Install it first: https://brew.sh"
                    exit 1
                fi
                ;;
        esac
    fi

    MPV_PATH=$(command -v mpv)
    if [ -n "$MPV_PATH" ]; then
        cp "$MPV_PATH" "$MPV_BIN"
        chmod +x "$MPV_BIN"
        echo "  -> $MPV_BIN (copied from $MPV_PATH)"
    else
        echo "ERROR: mpv installation failed or binary not found in PATH"
        exit 1
    fi
fi

echo ""
echo "All assets ready."

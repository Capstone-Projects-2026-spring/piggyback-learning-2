$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$ModelDir = Join-Path $RepoRoot "src-tauri\models"
$BinDir   = Join-Path $RepoRoot "src-tauri\binaries"

New-Item -ItemType Directory -Force -Path $ModelDir | Out-Null
New-Item -ItemType Directory -Force -Path $BinDir   | Out-Null

function Download-If-Missing {
    param([string]$Path, [string]$Url, [string]$Label)
    if (Test-Path $Path) {
        Write-Host "$Label already present, skipping"
    } else {
        Write-Host "Downloading $Label..."
        Invoke-WebRequest -Uri $Url -OutFile $Path -UseBasicParsing
        Write-Host "  -> $Path"
    }
}

$Triple = "x86_64-pc-windows-msvc"

# ── Whisper STT model ─────────────────────────────────────────────────────────
Download-If-Missing `
    -Path  "$ModelDir\ggml-base.en.bin" `
    -Url   "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin" `
    -Label "whisper base.en (~148MB)"

# ── WeSpeaker ONNX ────────────────────────────────────────────────────────────
Download-If-Missing `
    -Path  "$ModelDir\wespeaker.onnx" `
    -Url   "https://huggingface.co/Wespeaker/wespeaker-voxceleb-resnet34-LM/resolve/main/voxceleb_resnet34_LM.onnx" `
    -Label "wespeaker ONNX (~25MB)"

# ── Emotion ONNX ──────────────────────────────────────────────────────────────
Download-If-Missing `
    -Path  "$ModelDir\emotion-ferplus-8.onnx" `
    -Url   "https://github.com/onnx/models/raw/main/validated/vision/body_analysis/emotion_ferplus/model/emotion-ferplus-8.onnx" `
    -Label "emotion ONNX"

# ── Ultraface ONNX ────────────────────────────────────────────────────────────
Download-If-Missing `
    -Path  "$ModelDir\ultraface.onnx" `
    -Url   "https://github.com/Linzaer/Ultra-Light-Fast-Generic-Face-Detector-1MB/raw/master/models/onnx/version-RFB-320.onnx" `
    -Label "ultraface ONNX (~1MB)"

# ── yt-dlp ────────────────────────────────────────────────────────────────────
Download-If-Missing `
    -Path  "$BinDir\yt-dlp-$Triple.exe" `
    -Url   "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe" `
    -Label "yt-dlp ($Triple)"

# ── ffmpeg ────────────────────────────────────────────────────────────────────
Download-If-Missing `
    -Path  "$BinDir\ffmpeg-$Triple.exe" `
    -Url   "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-win32-x64" `
    -Label "ffmpeg ($Triple)"

Write-Host ""
Write-Host "All assets ready."

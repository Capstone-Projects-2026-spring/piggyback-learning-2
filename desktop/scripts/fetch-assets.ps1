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

Download-If-Missing `
    -Path  "$ModelDir\silero_vad.onnx" `
    -Url   "https://github.com/snakers4/silero-vad/raw/v5.1.2/src/silero_vad/data/silero_vad.onnx" `
    -Label "silero VAD ONNX v5 (~2MB)"

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

# ── Piper Alba voice model ────────────────────────────────────────────────────
Download-If-Missing `
    -Path  "$ModelDir\en_GB-alba-medium.onnx" `
    -Url   "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/en_GB-alba-medium.onnx" `
    -Label "Piper Alba voice (~60MB)"
Download-If-Missing `
    -Path  "$ModelDir\en_GB-alba-medium.onnx.json" `
    -Url   "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/en_GB-alba-medium.onnx.json" `
    -Label "Piper Alba config"

# ── Piper binary ──────────────────────────────────────────────────────────────
$PiperBin = "$BinDir\piper-tts-$Triple.exe"
if (Test-Path $PiperBin) {
    Write-Host "piper-tts already present, skipping"
} else {
    Write-Host "Downloading piper binary..."
    $PiperZip = "$env:TEMP\piper_windows.zip"
    Invoke-WebRequest `
        -Uri "https://github.com/rhasspy/piper/releases/latest/download/piper_windows_amd64.zip" `
        -OutFile $PiperZip -UseBasicParsing
    Expand-Archive -Path $PiperZip -DestinationPath "$env:TEMP\piper_extract" -Force
    Copy-Item "$env:TEMP\piper_extract\piper\piper.exe" $PiperBin
    # Copy required DLLs alongside the binary
    Copy-Item "$env:TEMP\piper_extract\piper\*.dll" $BinDir -ErrorAction SilentlyContinue
    Remove-Item $PiperZip, "$env:TEMP\piper_extract" -Recurse -Force
    Write-Host "  -> $PiperBin"
}

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

# ── mpv ───────────────────────────────────────────────────────────────────────
$MpvBin = "$BinDir\mpv-$Triple.exe"
if (Test-Path $MpvBin) {
    Write-Host "mpv already present, skipping"
} else {
    $mpvPath = $null
    try {
        $mpvPath = (Get-Command mpv -ErrorAction Stop).Source
        Write-Host "mpv found at $mpvPath"
    } catch {
        Write-Host "Installing mpv..."
        if (Get-Command winget -ErrorAction SilentlyContinue) {
            winget install --id=mpv.net -e --silent
            try { $mpvPath = (Get-Command mpv -ErrorAction Stop).Source } catch {}
        }
        if (-not $mpvPath -and (Get-Command scoop -ErrorAction SilentlyContinue)) {
            scoop install mpv
            try { $mpvPath = (Get-Command mpv -ErrorAction Stop).Source } catch {}
        }
        if (-not $mpvPath -and (Get-Command choco -ErrorAction SilentlyContinue)) {
            choco install mpv -y
            try { $mpvPath = (Get-Command mpv -ErrorAction Stop).Source } catch {}
        }
        if (-not $mpvPath) {
            Write-Host "ERROR: Could not install mpv automatically."
            Write-Host "Install manually: winget install mpv  OR  scoop install mpv"
            exit 1
        }
    }
    Copy-Item $mpvPath $MpvBin
    Write-Host "  -> $MpvBin (copied from $mpvPath)"
}

Write-Host ""
Write-Host "All assets ready."

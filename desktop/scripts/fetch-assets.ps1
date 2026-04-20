$ErrorActionPreference = "Stop"

# Always resolve paths relative to the repo root, not cwd
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot  = Split-Path -Parent $ScriptDir
$TauriDir  = Join-Path $RepoRoot "src-tauri"

Write-Host "==> Repo root: $RepoRoot"

$LIBVOSK_VER = "0.3.42"
$TRIPLE      = "x86_64-pc-windows-msvc"
$YT_DLP_VER  = (Invoke-RestMethod "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest").tag_name

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
function Test-NonEmptyFile($path) {
    (Test-Path $path) -and ((Get-Item $path).Length -gt 0)
}

function Test-NonEmptyDir($path) {
    (Test-Path $path) -and ((Get-ChildItem $path -Recurse -File | Measure-Object).Count -gt 0)
}

function Expand-AndCopyInner($zipPath, $destDir) {
    $tmp = "$env:TEMP\peppa-extract-$(Get-Random)"
    Expand-Archive -Path $zipPath -DestinationPath $tmp -Force
    $inner = Get-ChildItem -Path $tmp -Directory | Select-Object -First 1
    if ($inner) {
        Copy-Item "$($inner.FullName)\*" $destDir -Recurse -Force
    } else {
        Copy-Item "$tmp\*" $destDir -Recurse -Force
    }
    Remove-Item $tmp -Recurse -Force
}

# ---------------------------------------------------------------------------
# Directories
# ---------------------------------------------------------------------------
$dirs = @(
    "$TauriDir\binaries",
    "$TauriDir\models\vosk-spk-model",
    "$TauriDir\models\vosk-lgraph-model",
    "$TauriDir\models\mood",
    "$TauriDir\lib"
)
foreach ($d in $dirs) { New-Item -ItemType Directory -Force -Path $d | Out-Null }

# ---------------------------------------------------------------------------
# yt-dlp
# ---------------------------------------------------------------------------
$ytDlpDest = "$TauriDir\binaries\yt-dlp-$TRIPLE.exe"
if (Test-NonEmptyFile $ytDlpDest) {
    Write-Host "==> [skip] yt-dlp already present"
} else {
    Write-Host "==> Fetching yt-dlp ($YT_DLP_VER)..."
    Invoke-WebRequest `
        -Uri "https://github.com/yt-dlp/yt-dlp/releases/download/$YT_DLP_VER/yt-dlp.exe" `
        -OutFile $ytDlpDest
    Write-Host "   Done."
}

# ---------------------------------------------------------------------------
# ffmpeg
# ---------------------------------------------------------------------------
$ffmpegDest = "$TauriDir\binaries\ffmpeg-$TRIPLE.exe"
if (Test-NonEmptyFile $ffmpegDest) {
    Write-Host "==> [skip] ffmpeg already present"
} else {
    Write-Host "==> Fetching ffmpeg..."
    $ffmpegZip = "$env:TEMP\ffmpeg-win.zip"
    Invoke-WebRequest `
        -Uri "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip" `
        -OutFile $ffmpegZip
    $ffmpegExtract = "$env:TEMP\ffmpeg-win-extracted"
    Expand-Archive -Path $ffmpegZip -DestinationPath $ffmpegExtract -Force
    $ffmpegBin = Get-ChildItem -Path $ffmpegExtract -Recurse -Filter "ffmpeg.exe" |
        Select-Object -First 1
    Copy-Item $ffmpegBin.FullName $ffmpegDest
    Remove-Item $ffmpegZip, $ffmpegExtract -Recurse -Force
    Write-Host "   Done."
}

# ---------------------------------------------------------------------------
# Vosk speaker model
# ---------------------------------------------------------------------------
if (Test-NonEmptyDir "$TauriDir\models\vosk-spk-model") {
    Write-Host "==> [skip] Vosk speaker model already present"
} else {
    Write-Host "==> Fetching Vosk speaker model..."
    $zip = "$env:TEMP\vosk-spk.zip"
    Invoke-WebRequest `
        -Uri "https://alphacephei.com/vosk/models/vosk-model-spk-0.4.zip" `
        -OutFile $zip
    Expand-AndCopyInner $zip "$TauriDir\models\vosk-spk-model"
    Remove-Item $zip -Force
    Write-Host "   Done."
}

# ---------------------------------------------------------------------------
# Vosk lgraph model
# ---------------------------------------------------------------------------
if (Test-NonEmptyDir "$TauriDir\models\vosk-lgraph-model") {
    Write-Host "==> [skip] Vosk lgraph model already present"
} else {
    Write-Host "==> Fetching Vosk lgraph model..."
    $zip = "$env:TEMP\vosk-lgraph.zip"
    Invoke-WebRequest `
        -Uri "https://alphacephei.com/vosk/models/vosk-model-en-us-0.22-lgraph.zip" `
        -OutFile $zip
    Expand-AndCopyInner $zip "$TauriDir\models\vosk-lgraph-model"
    Remove-Item $zip -Force
    Write-Host "   Done."
}

# ---------------------------------------------------------------------------
# libvosk — Windows DLL
# ---------------------------------------------------------------------------
if ((Test-NonEmptyFile "$TauriDir\lib\libvosk.dll") -and (Test-NonEmptyFile "$TauriDir\lib\vosk_api.h")) {
    Write-Host "==> [skip] libvosk already present"
} else {
    Write-Host "==> Fetching libvosk $LIBVOSK_VER (Windows)..."
    $zip     = "$env:TEMP\libvosk-win.zip"
    $extract = "$env:TEMP\libvosk-win-extracted"
    Invoke-WebRequest `
        -Uri "https://github.com/alphacep/vosk-api/releases/download/v$LIBVOSK_VER/vosk-win64-$LIBVOSK_VER.zip" `
        -OutFile $zip
    Expand-Archive -Path $zip -DestinationPath $extract -Force
    $dll = Get-ChildItem -Path $extract -Recurse -Filter "libvosk.dll" | Select-Object -First 1
    $h   = Get-ChildItem -Path $extract -Recurse -Filter "vosk_api.h"   | Select-Object -First 1
    Copy-Item $dll.FullName "$TauriDir\lib\libvosk.dll"
    Copy-Item $h.FullName   "$TauriDir\lib\vosk_api.h"
    Remove-Item $zip, $extract -Recurse -Force
    Write-Host "   Done."
}

# ---------------------------------------------------------------------------
# ONNX mood model
# ---------------------------------------------------------------------------
if (Test-NonEmptyFile "$TauriDir\models\mood\emotion-ferplus-8.onnx") {
    Write-Host "==> [skip] ONNX mood model already present"
} else {
    Write-Host "==> Fetching ONNX mood model..."
    Invoke-WebRequest `
        -Uri "https://github.com/onnx/models/raw/main/validated/vision/body_analysis/emotion_ferplus/model/emotion-ferplus-8.onnx" `
        -OutFile "$TauriDir\models\mood\emotion-ferplus-8.onnx"
    Write-Host "   Done."
}

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
Write-Host ""
Write-Host "All assets ready. Bundle layout:"
Get-ChildItem -Path "$TauriDir\binaries","$TauriDir\models","$TauriDir\lib" `
    -Recurse -File | Select-Object -ExpandProperty FullName | Sort-Object

# Piggyback Learning (desktop)

A voice-activated educational desktop app for kids. Parents enroll their children by voice, assign YouTube videos, and the app quizzes kids at segment boundaries — tracking answers, mood, and engagement via webcam.

Built with **Tauri v2** (Rust) + **React** (Vite + Tailwind v4).

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [First-Time Setup](#first-time-setup)
- [Running in Development](#running-in-development)
- [Building for Production](#building-for-production)
- [Project Structure](#project-structure)
- [Wake Word](#wake-word)
- [Intent System](#intent-system)
- [Tauri Events](#tauri-events)
- [Database Schema](#database-schema)
- [TTS System](#tts-system)
- [Enrollment Flow](#enrollment-flow)
- [Voice Enrollment & Speaker Identification](#voice-enrollment--speaker-identification)
- [Key Dependencies](#key-dependencies)
- [Learn More](#learn-more)

---

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | stable | via [rustup](https://rustup.rs) |
| Node.js | 18+ | |
| npm | any | |
| piper-tts | latest | must be on system `PATH` |
| aplay | any | Linux only — part of `alsa-utils` |

> **macOS / Windows:** TTS playback (`aplay`) is Linux-only for now. The `speak` command is wired up but audio output on other platforms is not tested yet and may or may not work.

---

## First-Time Setup

### 1. Clone the repo

```bash
git clone https://github.com/your-org/piggyback-learning.git
cd piggyback-learning/desktop
```

### 2. Fetch models and binaries

This downloads all ONNX models, Whisper weights, and platform binaries (yt-dlp, ffmpeg, mpv). Run once before your first build.

**Linux / macOS:**
```bash
bash scripts/fetch-assets.sh
```

**Windows (PowerShell):**
```powershell
.\scripts\fetch-assets.ps1
```

The script places files into:
- `src-tauri/models/` — Whisper, WeSpeaker, UltraFace, emotion, TTS models
- `src-tauri/binaries/` — yt-dlp, ffmpeg, mpv (platform-suffixed sidecars)

It also installs `alsa-utils` on Linux (needed for `aplay`).

### 3. Configure environment

Copy the example Cargo config and fill in your API key:

```bash
cp src-tauri/.cargo/config.toml.example src-tauri/.cargo/config.toml
```

Then open `src-tauri/.cargo/config.toml` and set your OpenAI API key:

```toml
[env]
OPENAI_API_KEY = "sk-..."
```

### 4. Install frontend dependencies

```bash
npm install
```

### 5. Install piper-tts

Piper is not bundled as a Tauri sidecar yet — it must be available on your `PATH`.

```bash
# Example (Linux, adjust for your distro / release)
wget https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_linux_x86_64.tar.gz
tar -xzf piper_linux_x86_64.tar.gz
sudo mv piper/piper /usr/local/bin/piper-tts
```

Verify with:
```bash
piper-tts --version
```

---

## Running in Development

From the `desktop/` directory:

```bash
npm run tauri dev
```

This starts the Vite dev server and the Tauri app together. Hot reload is active for the React frontend.

---

## Building for Production

From the `desktop/` directory:

```bash
npm run tauri build
```

The installer/bundle will be placed under `src-tauri/target/release/bundle/`.

---

## Project Structure

```
desktop/
├── src/                   # React frontend
│   ├── components/        # UI, orb, enrollment, video, watch, questions, results
│   ├── hooks/             # useGazeTracker, usePlaybackPoller, useSegments, useTauriListener
│   ├── lib/stt/           # commandBus, recorder
│   └── utils/             # enrollment, orb, questions, tts, videoPanel helpers
├── src-tauri/
│   ├── .cargo/
│   │   └── config.toml.example   # copy to config.toml and add your API key
│   ├── src/               # Rust backend
│   │   ├── voice/         # Wake word, VAD, Whisper STT, speaker ID, intent pipeline
│   │   ├── handlers/      # Tauri command handlers (videos, questions, answers)
│   │   ├── utils/         # Gaze, mood, OpenAI client, crypto, text helpers
│   │   ├── db/            # SQLite init + schema
│   │   └── tts.rs         # Piper TTS integration
│   ├── models/            # ML models (populated by fetch-assets script)
│   └── binaries/          # yt-dlp, ffmpeg, mpv sidecars
└── scripts/               # fetch-assets.sh / .ps1
```

---

## Wake Word

Say **"Jarvis"** (also recognised: *Jarvas, Jarves, Jarvi, Jarviz*) to activate the voice pipeline.

### Example voice commands

| What you say | What it does |
|---|---|
| "Jarvis, search for videos about dinosaurs" | Searches YouTube |
| "Jarvis, add a new kid" | Starts kid enrollment |
| "Jarvis, show me Emma's results" | Opens results panel for Emma |
| "Jarvis, what are my videos" | Lists assigned videos for the active kid |
| "Jarvis, watch video" | Launches the current video in mpv |
| "Jarvis, download this video" | Starts the yt-dlp pipeline |

---

## Intent System

Intents are classified using [fastembed](https://github.com/Anush008/fastembed-rs). At startup, `init_classifier()` pre-embeds a set of example phrases for each intent. Incoming transcripts are embedded and compared via cosine similarity, with a keyword fallback for obvious patterns.

```rust
pub enum Intent {
    Search,
    AddKid,
    KidResults,
    AddTag,
    MyVideos,
    AssignVideo,
    WatchVideo,
    Recommendations,
    DownloadVideo,
    WakeOnly,
    Unhandled,
}
```

Resolved intents are dispatched to:

| Intent | Handler |
|--------|---------|
| `Search` | `handlers::videos::search` |
| `AddKid` | `handlers::kids::start_kid_enrollment` |
| `AddTag` | `handlers::kids::add_tags` |
| `MyVideos` | `handlers::kids::get_video_assignments` |
| `AssignVideo` | `handlers::kids::assign_video` |
| `Recommendations` | `handlers::kids::get_recommendations` |
| `KidResults` | `handlers::answers::get_kids_answers` (parent only — extracts kid name from transcript) |
| `WatchVideo` | emits `orb://watch-video` |
| `DownloadVideo` | handled by frontend |

---

## Tauri Events

Events flow from the Rust backend to the React frontend via Tauri's event system.

| Event | Payload | Purpose |
|-------|---------|---------|
| `orb://ready` | `{}` | App fully initialised |
| `orb://voice-result` | `{ transcript, wake_detected, command }` | All voice pipeline output |
| `orb://speaker-identified` | `{ user_id, name, role, score }` | After speaker ID |
| `orb://enrollment` | `{ flow, stage, message, prompts, prompt_index }` | Onboarding progress |
| `orb://search-status` | `{ status, query }` | Search in progress |
| `orb://search-results` | `{ results, query }` | Video search results |
| `orb://recommendations` | `{ kid_name, tags, recommendations }` | Kid recommendations |
| `orb://my-videos` | `{ videos }` | Assigned videos for kid |
| `orb://video-status` | `{ video_id, status }` | Download status |
| `orb://processing-status` | `{ video_id, stage, progress }` | Pipeline stage progress |
| `orb://questions-ready` | `{ video_id, segments }` | Questions generated |
| `orb://watch-video` | `{}` | Trigger video watch |
| `orb://mpv-tick` | `{ position }` | Playback position |
| `orb://segment-end` | `{}` | Segment boundary reached |
| `orb://answer-result` | `{ is_correct, similarity_score, mood, segment_id, transcript, video_id }` | Quiz answer evaluation |
| `orb://answers` | `{ answers, kid_id }` | All answers for a kid across all videos |
| `orb://answers-error` | `{ message }` | Kid name not resolved |
| `orb://gaze-status` | `{ status: "away" \| "present" }` | Gaze tracking |

---

## Database Schema

SQLite, managed via `sqlx`. The database is initialised on first launch.

```sql
users          (id, name, role, voice_embedding)
tags           (id, name)
kid_tags       (kid_id, tag_id)
videos         (id, title, thumbnail_url, duration_seconds, local_video_path)
video_tags     (video_id, tag_id)
video_assignments (kid_id, video_id, answers)
segments       (id, video_id, start_seconds, end_seconds, best_question)
questions      (id, segment_id, qtype, question, answer,
                followup_correct_question, followup_correct_answer,
                followup_wrong_question, followup_wrong_answer, rank)
answers        (id, kid_id, video_id, segment_id, transcript,
                is_correct, similarity_score, mood)
frames         (id, video_id, frame_number, timestamp_seconds,
                timestamp_formatted, filename, file_path, is_keyframe)
app_meta       (key, value)
```

---

## TTS System

Text-to-speech uses [Piper TTS](https://github.com/rhasspy/piper) with the `en_GB-alba-medium` voice.

When `speak` is called the backend sets `SessionMode::Tts`, spawns `piper-tts` piped to `aplay`, waits for a 300ms reverb buffer to clear, then resets back to `SessionMode::Command`. While in `Tts` mode the capture loop flushes its buffer and drops all incoming audio chunks, preventing the mic from picking up speaker output.

On the frontend, `utils/tts.js` always calls `stop_speaking` before `speak` to prevent overlapping playback.

---

## Enrollment Flow

Both parent and kid enrollment use the same `EnrollmentOverlay` component, driven by `orb://enrollment` events.

**Parent flow:** `greet` → `name_confirmed` → `prompt` ×N → `done`

**Kid flow:** identical stages on the Rust side, prefixed `kid_`. The frontend's `normaliseStage()` strips the prefix so both flows share the same component logic. Pass `flow="parent"` or `flow="kid"` to `EnrollmentOverlay` to control copy and styling.

The `commandBus.onEnrollment` handler caches the last parent enrollment event so that the overlay receives it correctly even if it mounts after the event was emitted.

---

## Voice Enrollment & Speaker Identification

### Enrollment

When a user enrolls, they are prompted to read 5 phrases aloud:

```
"The quick brown fox jumps over the lazy dog."
"She sells seashells by the seashore every summer."
"How much wood would a woodchuck chuck if a woodchuck could chuck wood?"
"Peter Piper picked a peck of pickled peppers."
"Around the rugged rocks the ragged rascal ran."
```

The recorded audio from each phrase is passed through the WeSpeaker ONNX model (`wespeaker.onnx`) to produce a speaker embedding from 80-mel filter bank features computed over the raw 16 kHz mono samples. That embedding is then encrypted and stored in the `users` table. Crucially, **the raw audio is never persisted** — only the embedding is stored, and only in encrypted form.

### Encryption

Voice embeddings are encrypted with **AES-256-GCM** before being written to the database.

AES-256-GCM is an authenticated encryption scheme, meaning it provides both confidentiality and integrity guarantees — an attacker who tampers with the ciphertext in the database will cause decryption to fail rather than silently produce garbage data. The 256-bit key size means there are 2²⁵⁶ possible keys, which is considered computationally infeasible to brute-force with any foreseeable hardware.

The encryption key is a **32-byte cryptographically random key** generated on first launch using the OS CSPRNG and stored in the **OS keychain** via the `keyring` crate:

| Platform | Storage |
|----------|---------|
| macOS | Keychain |
| Windows | Credential Manager |
| Linux | libsecret / GNOME Keyring |

This means the key is never written to disk in plaintext and is protected by the OS's own access control — other processes running as a different user cannot read it. On subsequent launches the key is loaded from the keychain and cached in a `OnceLock` for the lifetime of the process.

A **fresh 12-byte random nonce** is generated for every write using `OsRng`, so even if the same embedding were stored twice the ciphertexts would differ. The on-disk format stored in `users.voice_embedding` is:

```
[ 12-byte nonce ][ AES-256-GCM ciphertext + 16-byte auth tag ]
```

Decryption splits at byte 12, reconstructs the nonce, and verifies the auth tag before returning plaintext. If the tag check fails — whether due to corruption or tampering — decryption is aborted and the user is skipped during identification.

In practice this means that even if an attacker obtained a full copy of the SQLite database file, they would have an encrypted blob they cannot decrypt without the keychain key, which never leaves the OS secure store.

### Speaker Identification

At runtime, each captured audio chunk is run through the same WeSpeaker pipeline to produce a live embedding. That embedding is compared against every enrolled user by:

1. Fetching all rows from `users` where `voice_embedding IS NOT NULL`
2. Decrypting each blob with the cached key and verifying the auth tag
3. Deserialising the raw bytes back into a `Vec<f32>`
4. Computing **cosine similarity** between the live embedding and the stored one

The user with the highest similarity score wins. If that score is **≥ 0.75** the speaker is considered matched, the session is updated with their `user_id`, `name`, and `role`, and an `orb://speaker-identified` event is emitted to the frontend. If no user clears the threshold, the event is emitted with all fields set to `null`.

---

## Key Dependencies

**Rust**
- `tauri 2` — desktop shell
- `whisper-rs` — on-device speech-to-text (Whisper base.en)
- `ort` — ONNX runtime (speaker ID, face detection, emotion)
- `cpal` — cross-platform audio capture
- `sqlx` — async SQLite
- `async-openai` — question generation via GPT
- `aes-gcm` — AES-256-GCM voice embedding encryption
- `keyring` — OS keychain integration

**Frontend**
- React 19, Vite 6, Tailwind v4
- `@tauri-apps/api v2`

---

## Learn More

**Core framework**
- [Tauri v2](https://v2.tauri.app/start/) — desktop app framework
- [Tauri v2 Events](https://v2.tauri.app/develop/inter-process/events/) — Rust <-> frontend communication
- [Tauri v2 Commands](https://v2.tauri.app/develop/inter-process/commands/) — invoking Rust from JS

**Speech & audio**
- [Whisper](https://github.com/openai/whisper) — speech recognition model
- [whisper-rs](https://github.com/tazz4843/whisper-rs) — Rust bindings
- [WeSpeaker](https://github.com/wenet-e2e/wespeaker) — speaker embedding model
- [Piper TTS](https://github.com/rhasspy/piper) — text-to-speech engine
- [cpal](https://github.com/RustAudio/cpal) — cross-platform audio capture

**ML inference**
- [ONNX Runtime](https://onnxruntime.ai/docs/) — model inference
- [ort](https://github.com/pykeio/ort) — Rust bindings for ONNX Runtime
- [fastembed](https://github.com/Anush008/fastembed-rs) — intent embedding & classification

**Encryption**
- [AES-256-GCM (aes-gcm crate)](https://docs.rs/aes-gcm) — authenticated encryption
- [keyring](https://github.com/hwchen/keyring-rs) — OS keychain integration

**Database**
- [SQLx](https://github.com/launchbadge/sqlx) — async SQLite
- [SQLite](https://www.sqlite.org/docs.html) — embedded database

**Frontend**
- [React 19](https://react.dev/) — UI framework
- [Vite](https://vitejs.dev/) — frontend build tool
- [Tailwind CSS v4](https://tailwindcss.com/docs) — styling

**Media**
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) — video download
- [ffmpeg](https://ffmpeg.org/documentation.html) — frame extraction
- [mpv](https://mpv.io/manual/stable/) — video playback

**OpenAI**
- [async-openai](https://github.com/64bit/async-openai) — Rust OpenAI client
- [OpenAI API](https://platform.openai.com/docs/) — question generation

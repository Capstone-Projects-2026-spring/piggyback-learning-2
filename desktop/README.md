# Piggyback Learning (desktop)

A voice-activated educational desktop app for kids. Parents enroll their children by voice, assign YouTube videos, and the app quizzes kids at segment boundaries - tracking answers, mood, and engagement via webcam.

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
- [Tauri Commands](#tauri-commands)
- [Database Schema](#database-schema)
- [TTS System](#tts-system)
- [STT System](#stt-system)
- [VAD System](#vad-system)
- [Audio Processing](#audio-processing)
- [Enrollment Flow](#enrollment-flow)
- [Voice Enrollment & Speaker Identification](#voice-enrollment--speaker-identification)
- [Frontend / Startup Handshake](#frontend--startup-handshake)
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
| aplay | any | Linux only - part of `alsa-utils` |

> **macOS / Windows:** TTS playback (`aplay`) is Linux-only for now. The `speak` command is wired up but audio output on other platforms is not tested yet and may or may not work.

---

## First-Time Setup

### 1. Clone the repo

```bash
git clone https://github.com/your-org/piggyback-learning.git
cd piggyback-learning/desktop
```

### 2. Fetch models and binaries

Downloads all ONNX models and platform binaries (yt-dlp, ffmpeg, mpv). Run once before your first build.

**Linux / macOS:**
```bash
bash scripts/fetch-assets.sh
```

**Windows (PowerShell):**
```powershell
.\scripts\fetch-assets.ps1
```

The script places files into:
- `src-tauri/models/` - Moonshine, WeSpeaker, Silero VAD, UltraFace, emotion, TTS models
- `src-tauri/binaries/` - yt-dlp, ffmpeg, mpv (platform-suffixed sidecars)

It also installs `alsa-utils` on Linux (needed for `aplay`).

### 3. Configure environment

```bash
cp src-tauri/.cargo/config.toml.example src-tauri/.cargo/config.toml
```

Open `src-tauri/.cargo/config.toml` and set your OpenAI API key:

```toml
[env]
OPENAI_API_KEY = "sk-..."
```

### 4. Install frontend dependencies

```bash
npm install
```

### 5. Install piper-tts

Piper is not bundled as a Tauri sidecar yet - it must be available on your `PATH`.

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
│   ├── lib/               # commandBus, recorder (startOrb/stopOrb)
│   └── utils/             # enrollment, orb, questions, tts, videoPanel helpers
├── src-tauri/
│   ├── .cargo/
│   │   └── config.toml.example
│   ├── src/
│   │   ├── voice/         # Capture, VAD, Moonshine STT, speaker ID, intent pipeline
│   │   ├── handlers/      # Tauri command handlers (videos, questions, answers, kids)
│   │   ├── utils/         # Gaze, mood, OpenAI client, crypto, text helpers
│   │   ├── db/            # SQLite init + schema
│   │   ├── tts.rs         # Piper TTS integration
│   │   └── lib.rs         # App entry point, handshake, model loading
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

Intents are classified using [fastembed](https://github.com/Anush008/fastembed-rs). At startup, `init_classifier()` pre-embeds example phrases for each intent. Incoming transcripts are embedded and compared via cosine similarity, with a keyword fallback for obvious patterns.

| Intent | Example phrases |
|--------|----------------|
| `Search` | "search for dinosaur videos", "find videos about space", "look up minecraft videos", "show me videos about animals" |
| `AddKid` | "add a kid", "create a kid account", "enroll a kid", "make an account for my child", "add my son" |
| `KidResults` | "show me adam's results", "how did sarah do", "what were emma's answers", "pull up sarah's results" |
| `AddTag` | "my kid likes dinosaurs", "she loves painting", "he is really into space", "remember that i like cooking" |
| `MyVideos` | "my videos", "show my assigned videos", "what videos do i have", "list my assignments" |
| `AssignVideo` | "assign this to jake", "give it to emma", "this one is for jake", "assign this video to emma" |
| `WatchVideo` | "watch this", "play the first one", "let's watch this", "play the second one" |
| `Recommendations` | "show me recommendations for emma", "what should jake watch", "suggest videos for my son" |
| `DownloadVideo` | "download this", "save this video", "download video" |
| `WakeOnly` | wake word detected but no actionable intent |
| `Unhandled` | no intent matched above threshold |

Resolved intents are dispatched to:

| Intent | Handler |
|--------|---------|
| `Search` | `handlers::videos::search` |
| `AddKid` | `handlers::kids::start_kid_enrollment` |
| `AddTag` | `handlers::kids::add_tags` |
| `MyVideos` | `handlers::kids::get_video_assignments` |
| `AssignVideo` | `handlers::kids::assign_video` |
| `Recommendations` | `handlers::kids::get_recommendations` |
| `KidResults` | `handlers::answers::get_kids_answers` |
| `WatchVideo` | emits `orb://watch-video` |
| `DownloadVideo` | handled by frontend |

---

## Tauri Events

Events flow from the Rust backend to the React frontend via Tauri's event system.

| Event | Payload | Purpose |
|-------|---------|---------|
| `orb://ready` | `{}` | Backend fully initialised |
| `orb://voice-result` | `{ transcript, wake_detected, command }` | Voice pipeline output |
| `orb://speaker-identified` | `{ user_id, name, role, score }` | Speaker ID result |
| `orb://enrollment` | `{ flow, stage, message, prompts, prompt_index, total_prompts }` | Onboarding progress |
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
| `orb://answers` | `{ answers, kid_id }` | All answers for a kid |
| `orb://answers-error` | `{ message }` | Kid name not resolved |
| `orb://gaze-status` | `{ status: "away" \| "present" }` | Gaze tracking |

---

## Tauri Commands

Invoked from the frontend via `invoke()`.

| Command | Returns | Purpose |
|---------|---------|---------|
| `is_backend_ready` | `bool` | Poll until backend is fully initialised |
| `frontend_ready` | `bool` (needs_onboarding) | Signal backend that frontend has mounted; triggers onboarding if needed |
| `speak` | - | Speak text via Piper TTS |
| `stop_speaking` | - | Kill any active TTS immediately |
| `gaze_start` / `gaze_stop` | - | Start/stop webcam gaze tracking |
| `gaze_pause` / `gaze_resume` | - | Pause/resume gaze tracking |

---

## Database Schema

SQLite, managed via `sqlx`. The database is initialised on first launch.

```sql
users              (id, name, role)
voice_embeddings   (id, user_id, embedding)
tags               (id, name)
kid_tags           (kid_id, tag_id)
videos             (id, title, thumbnail_url, duration_seconds, local_video_path)
video_tags         (video_id, tag_id)
video_assignments  (kid_id, video_id, answers)
segments           (id, video_id, start_seconds, end_seconds, best_question)
questions          (id, segment_id, qtype, question, answer, followup_correct_question, followup_correct_answer, followup_wrong_question, followup_wrong_answer, rank)
answers            (id, kid_id, video_id, segment_id, transcript, is_correct, similarity_score, mood)
frames             (id, video_id, frame_number, timestamp_seconds, timestamp_formatted, filename, file_path, is_keyframe)
app_meta           (key, value)
```

`voice_embedding BLOB` has been removed from `users`. Embeddings are now stored individually in the `voice_embeddings` table - one row per enrollment phrase per user - and encrypted separately.

---

## TTS System

Text-to-speech uses [Piper TTS](https://github.com/rhasspy/piper) with the `en_GB-alba-medium` voice.

- Each call to `speak` kills any currently running `piper-tts` / `aplay` process before starting a new one, preventing overlapping playback
- A `Mutex` inside `TtsState` ensures only one TTS thread runs at a time
- `SessionMode::Tts` is set before piper spawns and cleared after a 300ms post-playback buffer
- While `SessionMode::Tts` is active, an `AtomicBool` gate in the mic callback discards all incoming audio before it ever enters the buffer - the capture loop has nothing to process
- The frontend always calls `stop_speaking` before `speak` as an additional safety net

---

## STT System

Speech-to-text uses **Moonshine base** (ONNX), loaded via `ort`. Four ONNX files are required:

```
models/moonshine-base/preprocess.onnx
models/moonshine-base/encode.onnx
models/moonshine-base/uncached_decode.onnx
models/moonshine-base/cached_decode.onnx
models/moonshine-base/tokenizer.json
```

The pipeline:
1. `preprocess` - converts raw 16 kHz mono f32 samples to mel features
2. `encode` - transformer encoder over mel features
3. `uncached_decode` - single forward pass with `SOT` token to get the first predicted token and initialise the KV cache
4. `cached_decode` - autoregressive decode loop using cached cross-attention

Token generation is capped at **6.5 tokens/second** of audio (minimum 10 tokens) to prevent hallucination loops. A **repetition detector** checks for repeating 4-token windows and truncates early if a loop is detected.

All output is lowercased and trimmed before being passed to the intent pipeline.

---

## VAD System

Voice Activity Detection uses **Silero VAD v5** (ONNX), loaded via `ort`.

- 512-sample frames at 16 kHz (32ms per frame)
- Speech probability threshold: **0.15**
- **8 consecutive silent frames** (~256ms) triggers a flush
- Maximum chunk duration: **15 seconds** (forced flush)
- Minimum chunk duration: **0.5 seconds** (shorter chunks discarded as noise)
- The Silero RNN hidden state is carried between frames within an utterance and reset on flush

---

## Audio Processing

The mic capture pipeline:

1. **`to_mono`** - average all channels to mono
2. **`resample`** - linear interpolation to 16 kHz if the device native rate differs
3. **`process_f32`** - peak normalize; reject if max amplitude < 1e-6 (silence)

Pre-emphasis and RNNoise denoising have been removed. Moonshine is a transformer trained on natural audio - pre-emphasis distorts the signal and causes mis-prediction and hallucination loops. Peak normalization alone produces the best transcription quality.

---

## Enrollment Flow

Both parent and kid enrollment use the same `EnrollmentOverlay` component. Enrollment events flow from Rust -> `orb://enrollment` -> `commandBus.dispatchEnrollment` -> `App.jsx` -> prop -> `EnrollmentOverlay`.

**Parent flow:** `greet` -> `name_confirmed` -> `prompt` ×5 -> `done`

**Kid flow:** identical stages, prefixed `kid_` on the Rust side. `normaliseStage()` strips the prefix so both flows share the same component logic.

Key implementation details:
- `App.jsx` is the single `commandBus.onEnrollment` subscriber - it stores the event in state and passes it down as a `currentEvent` prop, ensuring `speak()` is called exactly once per event with no duplicate TTS calls
- `startOrb()` (which registers Tauri event listeners) is called unconditionally in `App.jsx` so enrollment events are never missed even before `Orb.jsx` mounts
- The capture thread relies on `SessionMode::Tts` and the `AtomicBool` mic gate as the sole mechanism for suppressing audio during TTS - no fixed sleeps or energy thresholds

---

## Voice Enrollment & Speaker Identification

### Enrollment

Users are prompted to read 5 phrases:

```
"The quick brown fox jumps over the lazy dog."
"She sells seashells by the seashore every summer."
"How much wood would a woodchuck chuck if a woodchuck could chuck wood?"
"Peter Piper picked a peck of pickled peppers."
"Around the rugged rocks the ragged rascal ran."
```

Each phrase produces a **256-dimensional speaker embedding** via the WeSpeaker ONNX model, computed from 80-mel filter bank features over the raw 16 kHz mono audio. All 5 embeddings are stored individually in the `voice_embeddings` table - one encrypted row per phrase. Raw audio is never persisted.

### Speaker Identification

At runtime a live embedding is extracted from each captured chunk and compared against all enrolled users by:

1. Fetching all rows from `voice_embeddings` for each user
2. Decrypting and deserialising each embedding
3. Computing **cosine similarity** between the live embedding and each stored embedding
4. **Averaging** the similarity scores across all stored embeddings per user (using 3-second windows with 1-second hop for more stable results on longer chunks)

The user with the highest average score wins. Thresholds:
- Minimum score: **0.82**
- Minimum margin over second-best: **0.08**

### Encryption

Voice embeddings are encrypted with **AES-256-GCM** before being written to the database. A fresh 12-byte random nonce is generated per write. The on-disk format is:

```
[ 12-byte nonce ][ AES-256-GCM ciphertext + 16-byte auth tag ]
```

The 32-byte key is generated on first launch via the OS CSPRNG and stored in the OS keychain via the `keyring` crate:

| Platform | Storage |
|----------|---------|
| macOS | Keychain |
| Windows | Credential Manager |
| Linux | libsecret / GNOME Keyring |

The key never touches disk in plaintext. If an attacker obtains a copy of the SQLite database they have only encrypted blobs - decryption requires the keychain key which never leaves the OS secure store.

---

## Frontend / Startup Handshake

The backend emits `orb://ready` and sets an `is_backend_ready` flag when fully initialised. The frontend polls `is_backend_ready` on 100ms intervals, then calls `frontend_ready` once the backend is up.

`frontend_ready` returns `needs_onboarding: bool` - if true, the backend immediately starts parent onboarding and the frontend stays on the loading screen until the `greet` enrollment event arrives. If false, the frontend transitions directly to the ready state and speaks the welcome message.

This replaces the previous hardcoded 3-second delay.

---

## Key Dependencies

**Rust**
- `tauri 2` - desktop shell
- `ort` - ONNX runtime (Moonshine STT, Silero VAD, WeSpeaker, face detection, emotion)
- `cpal` - cross-platform audio capture
- `sqlx` - async SQLite
- `async-openai` - question generation via GPT
- `aes-gcm` - AES-256-GCM voice embedding encryption
- `keyring` - OS keychain integration
- `fastembed` - intent embedding & classification
- `tokenizers` - Moonshine tokenizer

**Frontend**
- React 19, Vite 6, Tailwind v4
- `@tauri-apps/api v2`

---

## Learn More

**Core framework**
- [Tauri v2](https://v2.tauri.app/start/)
- [Tauri v2 Events](https://v2.tauri.app/develop/inter-process/events/)
- [Tauri v2 Commands](https://v2.tauri.app/develop/inter-process/commands/)

**Speech & audio**
- [Moonshine](https://github.com/usefulsensors/moonshine) - speech recognition model
- [Silero VAD](https://github.com/snakers4/silero-vad) - voice activity detection
- [WeSpeaker](https://github.com/wenet-e2e/wespeaker) - speaker embedding model
- [Piper TTS](https://github.com/rhasspy/piper) - text-to-speech engine
- [cpal](https://github.com/RustAudio/cpal) - cross-platform audio capture

**ML inference**
- [ONNX Runtime](https://onnxruntime.ai/docs/)
- [ort](https://github.com/pykeio/ort) - Rust bindings for ONNX Runtime
- [fastembed](https://github.com/Anush008/fastembed-rs) - intent classification

**Encryption**
- [aes-gcm](https://docs.rs/aes-gcm) - authenticated encryption
- [keyring](https://github.com/hwchen/keyring-rs) - OS keychain integration

**Database**
- [SQLx](https://github.com/launchbadge/sqlx)
- [SQLite](https://www.sqlite.org/docs.html)

**Frontend**
- [React 19](https://react.dev/)
- [Vite](https://vitejs.dev/)
- [Tailwind CSS v4](https://tailwindcss.com/docs)

**Media**
- [yt-dlp](https://github.com/yt-dlp/yt-dlp)
- [ffmpeg](https://ffmpeg.org/documentation.html)
- [mpv](https://mpv.io/manual/stable/)

**OpenAI**
- [async-openai](https://github.com/64bit/async-openai)
- [OpenAI API](https://platform.openai.com/docs/)

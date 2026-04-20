```mermaid
graph TD
    subgraph Frontend ["Frontend (Next.js)"]
        direction LR
        UI["User Interface (App Router)"]
        WSH["WS Hook (Progress)"]
        AV["Video Player & Mascot"]
    end
    subgraph Comm ["Communications"]
        direction LR
        API["REST API (Axum)"]
        WSS["WebSockets (Axum)"]
        Assets["Static Assets Server"]
    end

    subgraph Backend ["Backend (Rust)"]
        direction TB
        Auth["Auth & Permissions"]
        Video_P["Video Processor (FFmpeg)"]
        AI["AI (Gemini)"]
        Vosk["Speech Recognition And Transcription (Vosk)"]
        Quiz["Quiz Logic"]
    end

    subgraph Storage ["Data"]
        direction LR
        DB[(SQLite / SeaORM)]
        FS["Local File System"]
    end

    subgraph External ["External APIs"]
        YT["YouTube API"]
        G_API["Gemini API"]
    end

    UI --> API
    WSH --- WSS
    AV <--- Assets

    API --- Auth
    API --- AI
    API --- Video_P
    API --- Vosk
    WSS --- Video_P

    Auth --- DB
    Video_P --- FS
    AI --- DB
    AI --- FS
    Vosk --- FS
    Assets --- FS

    Video_P --- YT
    AI --- G_API

    Quiz --- AI
    Vosk --- AI

Note, colored lines are just for readability purposes. They don't have any other meaning
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
        AI["AI (OPENAI)"]
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
        OPENAI_API["OpenAI API"]
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
    AI --- OPENAI_API

    Quiz --- AI
    Vosk --- AI
    linkStyle 3 stroke:#e69f00,stroke-width:3px;
    linkStyle 4 stroke:#56b4e9,stroke-width:3px;
    linkStyle 5 stroke:#cc79a7,stroke-width:3px;
    linkStyle 6 stroke:#3b82f6,stroke-width:3px;
    linkStyle 7 stroke:#d55e00,stroke-width:3px;
    linkStyle 8 stroke:#3b82f6,stroke-width:3px;
    linkStyle 9 stroke:#e69f00,stroke-width:3px;
    linkStyle 10 stroke:#cc79a7,stroke-width:3px;
    linkStyle 11 stroke:#d55e00,stroke-width:3px;
    linkStyle 12 stroke:#000000stroke-width:3px;
    linkStyle 13 stroke:#f0e442,stroke-width:3px;
    linkStyle 14 stroke:#d55e00,stroke-width:3px; 
    linkStyle 15 stroke:#f0e442,stroke-width:3px;
    linkStyle 16 stroke:#009e73,stroke-width:3px;
    linkStyle 17 stroke:#009e73,stroke-width:3px;
```

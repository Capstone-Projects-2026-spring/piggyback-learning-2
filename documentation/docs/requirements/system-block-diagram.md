```mermaid
graph TB
    subgraph Client_Layer ["Frontend (Next.js / React)"]
        UI["User Interface (App Router)"]
        WS_Client["WebSocket Hook (Real-time Progress)"]
        Media_Player["Video Player & Mascot Overlay"]
        Recorder["Audio Recorder (Vosk Integration)"]
    end

    subgraph API_Gateway ["Communication Layer"]
        REST["REST API (HTTP)"]
        WS_Server["WebSockets (Axum)"]
        Static["Static Asset Server"]
    end

    subgraph Logic_Layer ["Backend (Loco.rs / Rust)"]
        Auth_Service["Auth & Permissions"]
        Video_Proc["Video Processor (yt-dlp / FFmpeg)"]
        AI["AI (Gemini 2.5 Flash)"]
        Speech_Logic["Speech Transcription (Vosk)"]
        Quiz_Engine["Quiz Logic & Fallback"]
    end

    subgraph Data_Layer ["Persistence & Storage"]
        DB[(SQLite / SeaORM)]
        FileSystem["Local File System (Videos, Frames, JSON)"]
    end

    subgraph External_Services ["External Integrations"]
        YouTube["YouTube Content"]
        Gemini_API["Gemini AI (Question Gen)"]
    end

    %% Connections
    UI <--> REST
    WS_Client <--> WS_Server
    UI --- Media_Player
    
    REST --- Auth_Service
    REST --- Video_Proc
    REST --- AI
    
    Video_Proc --- YouTube
    Video_Proc --- FileSystem
    AI --- Gemini_API
    
    Speech_Logic --- FileSystem
    Quiz_Engine --- AI
    
    Auth_Service --- DB
    Video_Proc --- DB
    AI --- DB
    
    Static --- FileSystem
    Media_Player <--- Static
        

```mermaid
%%{init: {"flowchart":{"defaultRenderer":"elk"}}}%%
flowchart LR

  subgraph FE["Frontend Web Interface"]
    KidsUI["Kids Interface"]
    ExpertUI["Expert Review Interface"]
    AdminUI["Admin Control Panel"]
    Companion["Companion Selector"]
    QuizUI["Quiz and Feedback"]
    Rewind["Rewind or Continue Control"]

    KidsUI --> Companion
    Companion --> QuizUI
    QuizUI --> Rewind
  end

  subgraph BE["Backend FastAPI Server"]
    API["FastAPI Application"]
    KidsRoutes["Video and Quiz Routes"]
    AdminRoutes["Admin Routes"]
    WS["WebSocket Question Progress"]
    Frames["Frame Extraction OpenCV"]
    Parse["Transcript and Frame Parsing"]
    AI["AI Question Generation"]
    TTS["Text to Speech"]
    Auth["Password and Access Control"]
  end

  subgraph STORE["Local Storage"]
    Videos["Downloaded Videos and Metadata"]
    FramesStore["Extracted Frames"]
    AIQuestions["AI Generated Questions"]
    ExpertNotes["Expert Annotations"]
    FinalQ["Final Approved Questions"]
  end

  subgraph EXT["External Services"]
    YouTube["YouTube"]
    YTDLP["yt-dlp"]
    FFmpeg["FFmpeg"]
    Node["Node JS Optional"]
    OpenAI["OpenAI API"]
    ENV["Environment Variables"]
  end

  Child["Child User"] --> KidsUI
  Parent["Parent or Educator"] --> KidsUI
  Admin["Admin or Expert"] --> AdminUI
  Admin --> ExpertUI

  KidsUI --> API
  AdminUI --> AdminRoutes
  ExpertUI --> KidsRoutes

  API --> KidsRoutes
  API --> AdminRoutes
  API --> TTS

  AdminRoutes --> WS
  AdminRoutes --> YTDLP
  AdminRoutes --> Frames
  AdminRoutes --> Parse
  AdminRoutes --> AI

  YTDLP --> YouTube
  YTDLP --> FFmpeg
  YTDLP --> Node
  YTDLP --> Videos

  Frames --> FramesStore
  Parse --> FramesStore
  AI --> OpenAI
  OpenAI --> AIQuestions
  TTS --> OpenAI

  KidsRoutes --> ExpertNotes
  KidsRoutes --> FinalQ

  ENV --> API
  ENV --> OpenAI
  Auth --> API

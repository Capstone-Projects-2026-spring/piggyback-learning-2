---
sidebar_position: 2
---

# System Block Diagram
```mermaid
    flowchart TD
        Admin["Admin User Browser"]
        Admin --> ARouter["Admin Router <br/> (admin_routes)"]
        ARouter --> ATubeDownload["Youtube Downloader <br/>(yt-dlp)"]
        ATubeDownload --> AFrameExtractor["Frame Extractor <br/>OpenCV to extracted_frames/(JPEG, CSV,JSON)"]
        AFrameExtractor --> AQuestGen["Question Generator <br/> OpenAI WebSocket"]
        AQuestGen --> AVidSave["Save to <br/> Database <br/> Video files, meta.json, subtitles "]
        AVidSave --> AQuestSave["Save questions to Database <br/> in questions/"]

        Expert["Expert User Browser"]
        Expert --> EEndpoints["Expert Endpoints <br/> /expert-preview, /api/tts"]
        EEndpoints --> EQuestLoad[" Load questions from Database <br/> questions/ "]
        EQuestLoad --> ESaveEditQuest["Save expert edited questions to Database <br/> /api/expert-annotations, expert_questions/"]
        ESaveEditQuest --> ESaveFinal["Finalize and save questions to Database <br/> /api/save-final-questions, final_questions/ "]

        Child["Child User Browser"]
        Child --> CVidRouter["Video Quiz Router <br/> video_quiz_routes.py"]
        CVidRouter --> CLoadQuest["Load final questions from Database <br/> final_questions/"]
        CLoadQuest --> CVidQuiz["Play video with quiz <br/> Whisper TTS"]
        CVidQuiz --> CAnswer["Answer Submission and Grading<br/> Whisper speech‑to‑text"]
        
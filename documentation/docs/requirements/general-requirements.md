---
sidebar_position: 3
---

# General Requirements

## General Requirements For Users
1. Desktop or laptop
2. Internet access
3. Open AI API key


## Project Requirments
### Technology Stack Requirements
1. Backend Framework: The backend must be developed using Rust and the Loco.rs framework (using Axum for routing).
2. Database & ORM: Data must be handled by SQLite using SeaORM.
3. Frontend Framework: The user interface must be built with Next.js (React) using the App Router and TypeScript.
4. Styling: UI styling must be implemented using Tailwind CSS.
5. Communication: Video processing must be implemented using WebSockets.

### Media & AI Requirements
1. Video Processing: The system must use yt-dlp for YouTube ingestion and FFmpeg for frame extraction and video manipulation.
2. Generative AI: Question generation and answer validation must utilize OpenAI.
3. Speech-to-Text (STT): Child response audio transcription must be performed using the Vosk library (locally). This is to ensure privacy to comply with COPPA and GDPR-K laws.
4. Hardware Access: The frontend must have authorization to access the user's camera (for eye tracker) and microphone (for quiz responses).

### Platform and Hardware
1. Browser Support: The application must function properly on modern browsers, including but not limited to Chrome, Edge, and Safari.
2. Device Requirements: The user's device must have a camera and a microphone.
3. Operating Systems: The backend must be compatible with Linux and macOS.

### Legal and Regulatory Standards
Since the main users are young children, the app must be developed with very high privacy standards.
1. COPPA Compliance: The app must comply with the Children's Online Privacy Protection Act (COPPA) for users in the US. Specifically:
 - No raw voice audio shall be stored permanently on servers.
- Eye-tracking data processing must be local.
2. GDPR-K: The app must comly to GDPR-K (General Data Protection Regulation for children) for users in the EU, requiring consent from the parent for data processing.
3. Data Security: All user data inside the database must be encrypted.

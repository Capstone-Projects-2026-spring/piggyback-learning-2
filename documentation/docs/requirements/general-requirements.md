---
sidebar_position: 3
---

# General Requirements

## System Requirements for Users
- **Hardware:** Desktop or laptop with a front-facing camera and a microphone.
- **Connectivity:** Minimum stable internet connection for YouTube streaming and AI API calls.
- **Configuration:** A valid OpenAI API key must be provided in the environment settings.

## Project Requirements

### Technology Stack Requirements
1. **Backend Framework:** Must be developed using **Rust** and the **Loco.rs** framework (utilizing **Axum**).
2. **Persistence:** Must utilize **SQLite** with **SeaORM** as the Object-Relational Mapper.
3. **Frontend:** Must be built with **Next.js** (React) using the **App Router** and **TypeScript**.
4. **Styling:** Must use **Tailwind CSS** for responsive design.
5. **Real-time Protocol:** Video processing progress and distraction alerts must use **WebSockets**.

### Media and AI Requirements
1. **Utility Tools:** Must use `yt-dlp` for video ingestion and `FFmpeg` for frame extraction.
2. **AI Engine:** Question generation and answer grading must utilize **OpenAI** large language models.
3. **Privacy Based STT:** Speech-to-text must be performed using the **Vosk** library running locally to maintain COPPA compliance.
4. **Hardware Permissions:** Frontend must request and receive browser-level authorization for `camera` and `microphone` access.

### Platform Compatibility
1. **Browser Support:** Must support the latest versions of **Chrome**, **Edge**, and **Safari**.
2. **Hardware Support:** Must function on any device providing a camera and audio inputs.
3. **Server OS:** Backend source code must compile and run on **Linux** and **macOS**.

### Legal and Regulatory Standards
Since the main users of the app are young children, the app must be developed with very high privacy standards.
1. **COPPA/GDPR-K:** - No persistence of raw audio data.
    - Eye tracking (biometric) data must be processed in the browser's memory and never transmitted.
2. **Parental Consent:** The system must implement a signup procedure where child accounts are created only under a parent's authenticated session.
3. **Encryption:** All database values containing Personal Identifiable Information, such as usernames, must be encrypted.
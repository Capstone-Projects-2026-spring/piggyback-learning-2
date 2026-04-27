---
sidebar_position: 1
---

# System Overview

## Project Abstract

This document proposes improvements to an existing application, Piggyback Learning. This application aims to help children retain focus while watching educational videos by automatically generating quizzes based on the video content. As the video plays, it will pause at selected intervals to play a short quiz.

### High Level Requirements
The system must implement the following improvements:
- **Mascot Interaction:** An animated character overlay that provides audio and visual feedback to children.
- **Data Analytics:** Collection and of data such as: quiz accuracy, average mood, and quiz answer results.
- **No Distraction Mode:** A restricted playback that disables navigation controls.
- **Enhanced Speech Processing:** Integration of local STT (Vosk) and fuzzy-matching logic for child-specific speech patterns.
- **Question Scaffolding:** A multi-tiered fallback system where incorrect answers trigger guided, simpler questions.
- **Low-Latency Architecture:** Performance targets for AI generation and data retrieval.
- **Engagement Monitoring:** Computer-vision-based detection of child distraction events.

## Conceptual Design

Piggyback Learning is a Next.js and Rust web application that automates the transformation of YouTube content into interactive lessons. It utilizes `yt-dlp` for ingestion, `FFmpeg` for visual analysis, and OpenAI's GPT-4o-mini (via Gemini 2.5 Flash protocols) to generate age-appropriate educational content.

## Background
Piggyback Learning is designed for children ages 4-7. The system turns passive video viewing into active learning by requiring them to demonstrate comprehension before the video continues.

### Key Features and Differentiators:
- **Automated AI Question Generation:** Eliminates the need to manually create quizzes by analyzing video transcripts and frames to generate questions automatically via AI.
- **Voice Interaction:** Quiz response input is audio-based, utilizing local STT to ensure privacy and accessibility for illiterate users.
- **Real-Time Gaze Tracking:** Uses client side computer cameras to identify distraction events (glancing away from computer) and pause playback.
- **Adaptive Scaffolding:** Dynamic quiz logic that adapts to user errors by providing questions that lead to the correct answer rather than simple Correct/Incorrect feedback.

### Comparison to Existing Products

| Feature | Piggyback Learning | Edpuzzle | Khan Academy | Nearpod |
| :--- | :--- | :--- | :--- | :--- |
| **Quiz Creation** | AI-Automated | Manual | Pre-made/Static | Manual |
| **Input Method** | Voice/Audio | Text/Multiple Choice | Text/Multiple Choice | Text/Interactive |
| **Attention Tracking** | Eye Detection | None | None | Engagement Reports |
| **Target User** | Parent/Home | Teacher/Classroom | Student/Self-paced | Teacher/Classroom |

### Child Privacy and Legal Compliance
The system is designed to meet **COPPA** (USA) and **GDPR-K** (EU) standards through the following technical constraints:
- **Local STT:** Audio transcription (Vosk) occurs entirely on the local machine; raw audio files are not stored on the server.
- **Client-Side Vision:** Eye detection (MediaPipe) processes frames locally in the browser; camera feeds are never uploaded or stored.
- **Anonymized Data:** Stored performance metrics are accessible only to the parent account linked via a secure unique identifier.

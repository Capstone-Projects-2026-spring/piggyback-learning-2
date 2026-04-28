---
sidebar_position: 1
---

# System Overview

## Project Abstract

This document proposes improvements to an existing application, Piggyback Learning. This application aims to help children retain focus while watching educational videos by automatically generating quizzes based on the video content. As the video plays, it will pause at selected intervals to play a short quiz.

### High-Level Requirements
The system must implement the following improvements:
- **Mascot Interaction:** An animated mascot character overlay that gives audio and visual feedback to children.
- **Data Analytics:** Collection of data such as: quiz accuracy, average mood, and quiz answer results.
- **No Distraction Mode:** A restricted playback that disables navigation controls.
- **Enhanced Speech Processing:** Integration of local STT (Vosk) and fuzzy-matching logic for child speech.
- **Question Scaffolding:** A fallback system where incorrect answers trigger guided questions to correct answers.
- **Low-Latency Architecture:** Performance speed increase for AI generation and data retrieval.
- **Engagement Monitoring:** Device camera detection of child distraction events.

## Conceptual Design

Piggyback Learning is a Next.js and Rust web application that automates the transformation of YouTube content into learning. It uses `yt-dlp` for video downloads, `FFmpeg` for analysis, and OpenAI to generate age-appropriate educational questions.

## Background
Piggyback Learning is designed for children ages 4-7. The system turns regular video watching into learning by requiring them to demonstrate comprehension before the video continues.

### Key Features and Differentiators:
- **Automated AI Question Generation:** Removes the need to manually create quizzes by analyzing video transcripts and frames to generate questions automatically via AI.
- **Voice Interaction:** Quiz response input is audio, using local STT to ensure privacy and accessibility for illiterate young users.
- **Real-Time Gaze Tracking:** Uses client-side computer cameras to identify distraction events (glancing away from the computer) and pause playback.
- **Adaptive Scaffolding:** Adaptable quiz logic that adapts to users when they answer questions wrong by providing questions that lead to the correct answer rather than simple Correct/Incorrect feedback.

### Comparison to Existing Products

| Feature | Piggyback Learning | Edpuzzle | Khan Academy | Nearpod |
| :--- | :--- | :--- | :--- | :--- |
| **Quiz Creation** | AI-Automated | Manual | Pre-made/Static | Manual |
| **Input Method** | Voice/Audio | Text/Multiple Choice | Text/Multiple Choice | Text/Interactive |
| **Attention Tracking** | Eye Detection | None | None | Engagement Reports |
| **Target User** | Parent/Home | Teacher/Classroom | Student/Self-paced | Teacher/Classroom |

### Child Privacy and Legal Compliance
All applications that may store and process data relating to children must comply with **COPPA** (USA) and **GDPR-K** (EU) laws. Especially if the applications use AI. The app must obtain clear consent from the parents before collecting, using, or disclosing data of their children. The app must implement strong security for data and minimize data collection.
The system is designed to meet **COPPA** and **GDPR-K** standards through the following constraints:
- **Local STT:** Audio transcription (Vosk) occurs entirely on the local machine. Raw audio files are not stored on the server.
- **Client-Side Vision:** Eye detection (MediaPipe) processes frames locally in the browser. Camera feeds are never uploaded or stored.
- **Anonymized Data:** Stored performance metrics are accessible only to the parent account linked via a secure, unique identifier.

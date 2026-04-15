---
sidebar_position: 4
---

# Features and Requirements

## Features
- Voice recognition: App should successfully listen and understand the child's voice.
- Mood Detection: App checks for engagement by detecting if the child is looking at the video.
- Quiz Fallback: Alternative fallback options are provided if the child answers questions wrong. (rewatching, and scaffolding) to assist in correctly answering the question.
- No Distraction Mode: Allow parents to block child from using certain video features which may distract them.
- Question Scaffolding: If a child answers questions incorrectly, the application will generate leading questions to guide the child toward the correct answer.
 - User Data Dashboard: A dashboard that allows parents to see watchtime, performance, and other data about their child.
 - Mascot: A character overlay that reads aloud questions and tells the child if they got the question correct.
 - Account Management & Parental Controls: Tiered permissions that allows parents to assign videos to kids and monitor them.

## Functional Requirements

### Voice Recognition
- Convert speech input to text for processing.
- Mascot uses text-to-speech (TTS) to 'read' questions and answer choices aloud.

### Mood Detection
- Utilizes facial recognition to determine if the child's eyes are directed at the video.
- If App detects the child is distracted, then the video will be paused.
- If a distraction is detected, the App sends a notification to the parent.

### Quiz Fallback And Logic
- The App may request AI-generated scaffolded questions when a child answers incorrectly. Under control of parent.
- The App may rewind a section of the video if the child answers incorrectly.
- AI-generated questions must be reviewed by a human/parent before they are availaible to children.


### No Distraction Mode
- Parent accounts can toggle "No Distraction Mode" ON/OFF for their child accounts.
- When "No Distraction Mode" is active
	- hyperlinks are disabled within the video.
	- fast forward and rewind are blocked within the video.

### User Data
- There should be a dashboard with sections named:
	- Summary
	- User Management
	- Settings.
- User data must be securely logged and saved to the database.

### Mascot
- Mascot must be overlayed over videos
- Mascot must 'read' the questions and answers outloud.

### Account
- Users must be able to create an with a unique username, secure password, and accountID
- Various Permission levels must exist.  Parent account type must be able to assign specific video access to a Child account type, along with controlling "No Distraction Mode".

### Video processing
- Use a YouTube URL provided by the user and process the video file using yt-dlp and FFmpeg to prepare it for the internal player.

## Nonfunctional Requirements

### Performance & Latency
- overall interaction response time must be improved. (Including the AI response latency speed which shall be improved by 30%)
- Data retrieval from the database should be fast (perceived as instantaneous)
- Data should be saved within a reasonable time.

### Accuracy & Reliability
- Voice recognition must accurately interpret child voice in varying circumstances, including child speech patterns, different accents, and loud volume.
- The App should map voice response transcribed to text to the correct answer, even if not fully correct.
- The App shall be accurate when distinguishing Distracted and Not Distracted
- Improvements to voice recognition:
	
### Compliance to Security & Privacy
- No raw voice data recordings shall be stored, in compliance with child privacy laws (COPPA/GDPR-K).
- All stored user data shall be encrypted and private, accessible only by the specific child account and its linked parent account.
- No Distraction Mode shall not easily bypassed by a child user
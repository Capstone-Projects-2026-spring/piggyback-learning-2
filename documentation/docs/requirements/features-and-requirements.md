---
sidebar_position: 4
---

# Features and Requirements

## Functional Requirements

### Voice Recognition

- Voice recognition should be able to listen and understand users.
- Voice recognition may convert speech to text

### Mood Detection

- Application should be able to detect mood of child based on voice
- Application should have fallback options if it detects child is distracted.

### Quiz Fallback

- If the user answers a quiz question(s) wrong, there should be fall back options.
- Fall back options may include multiple choice questions, replaying a part of the video , or question layering.
	- Application must ask AI to generate multiple choice questions and question Scaffolding
	- Humans must review questions before they are available to users.
- Additional spotlight mode fallback. Video will rewind and a spotlight will shine to show where correct answer is.

### No Distraction Mode

- Parent account should be able to toggle No Distraction Mode for videos on the kids account.
- No Distraction Mode should:
	- turn off hyperlinks for the video
	- restrict fast forwarding
	- restrict rewinding
	- not allow jumping around the video.

### Question Scaffolding

- If child answers question wrong then AI will generate additional questions.
- The questions will slowly lead child to correct answer to original question, after responses.

### User Data

- User data should be saved and securely logged in the database (user id, video id, watch time, correct / incorrect answer)
- There should be a dashboard that contains:
	- Summary
	- Users
	- Settings

### Mascot

- Mascot must be overlayed over videos
- Mascot must 'read' the questions and answers outloud.

### Account

- Users must be able to create their own accounts (username, password, accountID)
- Users must have different permission levels that will give them access to which videos they can watch. Think parental controls: the parent’s account can control what the kids account can / cannot watch.

## Nonfunctional Requirements

- Improvements to voice recognition:
	- Voice recognition needs to be able to interpret less ‘perfectly worded’ input (ie the screaming of a child, different accents, etc)
	- Should be able to map input to answers, even if they are not the same 
- Response time needs to be improved on
- No voice data will be stored, in order to comply with the law (children voice)
- All data that may be stored shall be secure and private, only available to the kid account and the associated parent account.
- Improve AI response latency speed by 30%
- AI should able to generate good questions for Question Scaffolding quickly and on the fly.
- Data should be saved within a reasonable time frame.
- Data retrieval from database should be fast.
- Application should accurately detect mood from the two options 'Distracted' and 'Not Distracted' with a 5% margin of error.
- No Distraction Mode should accomplish it's goals of keeping the child on track and shouldn't be easily bypassed.
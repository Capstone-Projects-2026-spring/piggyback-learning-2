---
sidebar_position: 4
---

# Features and Requirements

## Functional Requirements

### Account

- Users must be able to create their own accounts (username, password, accountID)
- Users must be able to login, logout and change their passwords.
- Users must have different permission levels that will give them access to which videos they can watch. Think parental controls: the parent’s account can control what the kids account can / cannot watch.

### Quiz Fallback

- If the user answers a quiz question(s) wrong, there should be fall back options.
- Fall back options may include multiple choice questions, replaying a part of the video , or question layering.
	- Application must ask AI to generate multiple choice questions and question layering
	- Humans must review questions before they are available to users.
- Additional spotlight mode fallback. Video will rewind and a spotlight will shine to show where correct answer is.

### User Data

- User data should be saved and securely logged in the database (user id, video id, watch time, correct / incorrect answer)
- There should be a dashboard that contains:
	- Summary
	- Users
	- Settings

### No Distraction Mode

- Parent account should be able to toggle No Distraction Mode for videos on the kids account.
- No Distraction Mode should:
	- turn off hyperlinks for the video
	- restrict fast forwarding
	- restrict rewinding
	- not allow jumping around the video.

### Mascot
- Mascot overlay over videos
- Mascot will 'read' the questions and answers outloud.

### Voice Recognition
- Voice recognition should be able to listen and understand users.
- Voice recognition may convert speech to text
- Voice recognition should be able to detect if the child is paying attention and have some fallback options.

## Nonfunctional Requirements

- Improvements to voice recognition:
	- Voice recognition needs to be able to interpret less ‘perfectly worded’ input (ie the screaming of a child, different accents, etc)
	- Should be able to map input to answers, even if they are not the same 
- Response time needs to be improved on
- No voice data will be stored, in order to comply with the law (children voice)
- All data that may be stored shall be secure and private, only available to the kid account and the associated parent account.
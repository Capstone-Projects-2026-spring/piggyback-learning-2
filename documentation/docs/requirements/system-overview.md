---
sidebar_position: 1
---

# System Overview

## Project Abstract

This document proposes improvements to an existing application, Piggyback Learning. This application aims to help children retain focus while watching educational videos by automatically generating quizes based on the video they’re watching. As the video plays, it will occasionally pause while a short quiz about the events of the video is queued.

### High Level Requirement
The improvements include:
- Mascot animation and overlays
- User statistic gathering and display
- A toggleable No Distraction Mode.
- Better voice recognition
- Question Layering
- More fallback options
- Faster speed
- Attention/Mood detection

## Conceptual Design

Piggyback Learning is a FastAPI-based web application that downloads YouTube videos, extracts frames, and generates educational comprehension questions for children using OpenAI's GPT-4 Vision API. 

## Background
Piggyback Learning is an interactive educational platform designed for children ages 4-7. The App alters passive video consumption into active learning by auto generating questions that appear at intervals during video playback. Unlike normal video platforms where children simply watch videos, Piggyback Learning requires the child to demonstrate understanding of the video before proceeding.

The system takes links to YouTube videos, processes them for playback, and leverages AI to analyze video frames and generate age-appropriate educational questions. 

Key differences include:
- AI-Powered Question Generation: Automatic creation of questions that are specifically tailored to video content and children ages
- Voice Interaction: Speech recognition allows children to answer questions verbally
- Mascot Overlay: An animated character that reads questions aloud and guides children to answer correctly
- No Distraction Mode: Parental controls that restrict navigation, fast-forwarding, and rewinding to help children keep focused
- Mood/Attention Detection: Facial recognition technology monitors whether the child is looking at the screen and notifies the parent if the child is distracted
- Question Scaffolding: After child answers incorrectly, the App generates questions to guide them towards the correct answer
- Parent Dashboard: Contains various data on watch time, quiz performance, and more.

### Comparison to Existing Products
#### Edpuzzle
Edpuzzle is a popular tool that allows teachers to embed questions, notes, and audio commentary into existing videos from various platforms. Teachers create questions manually and can track student progress through a gradebook dashboard. Edpuzzle allows instructors to prevent students from skipping videos, and review their responses.

##### Similarities to Piggyback Learning:
- They both alter passive video watching into interactive learning
- They both include a "prevent skipping" functionality (No Distraction Mode is our equivalent)
- They both include multiple formats for questions and allows tracking of student performance

##### Differences:
- Edpuzzle requires questions to be manually created by teachers. Piggyback Learning automatically generates questions using AI.
- Edpuzzle doesn't have the capabilities for voice-based responses
- Edpuzzle does not have mood/attention detection and animated mascots

#### Khan Academy
Khan Academy has a large library of educational videos, practice exercises, and mastery-based learning paths. The platform is free and includes official test prep for SAT, AP, and other standardized exams. Khan Academy Kids targets children with content focused on various subjects such as literacy, math, and social-emotional learning .

##### Similarities to Piggyback Learning:
- They both are for early childhood education
- They both use video content with interactive questioning
- They both provide progress tracking for parents

##### Differences:
- Khan Academy uses a huge library of content created by educators. Piggyback Learning allows any YouTube video to be transformed into a learning experience.
- Khan Academy contents are pre-made, fixed, and static. Piggyback Learning auto generates questions based on video content
- Khan Academy's questions are mostly multiple-choice and text input. Piggyback Learning only uses voice interaction for questions.
- Khan Academy Kids includes a mascot but does not overlay them on third-party videos like Piggyback Learning

#### Nearpod
Nearpod is a learning platform that allows teachers to create or import presentations and embed formative assessments, polls, VR field trips, 3D objects, and collaborative activities . Teachers teach the lessons synchronously with live participation or asynchronously. The platform includes a large library of lessons and supports integration with Google Slides and many other Learning Management Systems .

##### Similarities to Piggyback Learning:
- They both support embedding interactive questions within videos
- They both provide real-time feedback based on student responses
- They both have capabilities to monitor engagement of users (Mood/Distraction detection)

##### Differences:
- Nearpod is designed for classroom instruction facilitated by teachers. Piggyback Learning is designed for independent use at home supervised by parents.
- Nearpod requires lessons to be manually created and uses pre-built libraries. Piggyback Learning offers automated, on-the-fly question generation.
- Nearpod doesn't have attention detection and parental control features. (like No Distraction Mode)

#### Wayground
Wayground is a browser extension and web application that generates interactive activities, quizzes, and flashcards from pre-existing content which may include websites, YouTube videos, Google Docs, and AI-generated text . Users can easily use Wayground to create assessments, and other comprehension checks from text or videos. The platform includes features such as auto-grading, accommodations, and reporting features.

##### Similarities to Piggyback Learning:
- They both auto generate questions from videos using AI
- They both use Youtube videos as a main source of content
- They both have real-time assessment and feedback features

##### Differences
- Wayground primary focus is one teacher workflows and classroom integration while Piggyback Learning is more focused on parent-controlled, at home learning
- Wayground doesn't have voice-based response capabilities
- Wayground does not include attention detection capabilties or mascot overlays
- Wayground doesn't have navigation restrictions on videos, which Piggyback Learning has.

### Child Privacy, COPPA Compliance, GDPR-K Compliance
Any application collecting personal information from children under 13 in the United States must comply with the Children's Online Privacy Protection Act (COPPA). And any app collecting personal information from children in the EU must comply with General Data Protection Regulation for children (GDPR-K). This includes requirements for parental consent, clear privacy notices, and limitations on data collection and retention.

#### Implications for Piggyback Learning
 The application's voice interaction and attention detection features must be considered in terms of COPPA. No voice data will be stored, and all data collection will be limited to anonomous usage data that are only accessible to linked parent accounts and children accounts, in compliance with COPPA.

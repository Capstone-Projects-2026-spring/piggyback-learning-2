---
sidebar_position: 1
---

# System Overview

This document proposes an application that allows young children to retain the attention and comprehension of children while they consume the videos that they watch on a daily basis. The application offers activities that consist of a YouTube video and quiz questions about the video, all prompted throughout the its duration. Children are to answer the questions using their voice. There are settings to let users decide whether the video continues or rewinds when a question is answered incorrectly. The application records and tracks users' individual progress, saving data such as watchtime, correctly and incorrectly answered  questions, and video data. 

# Conceptual Design

The system is divided into a frontend and a backend that work together to deliver an interactive, video-based learning experience designed for children.

The frontend presents YouTube videos, displays quiz questions at predefined points during playback, and allows the child to select a companion character with a distinct personality and teaching style. The chosen companion influences how explanations, hints, and questions are presented. The frontend provides immediate feedback, optional video rewind after incorrect answers, and displays progress information in a child-friendly interface.

The backend handles all processing and data management. It stores video activities, questions, and companion-related configurations, evaluates user responses, determines correctness, and tracks individual user progress such as quiz results and completion history. The backend exposes APIs that allow the frontend to retrieve learning activities, submit responses, manage user IDs, and securely store progress data for later review by parents or educators.

# Background

[EdPuzzle](https://edpuzzle.com/) is a similar application. It also incorporates YouTube videos to promote learning. The website gives options to create video quizzes with multiple choice answers and even voice-recorded responses. The videos are linked and questions are added by teachers, with features to grade student responses and connect to LMS. However, Piggyback focuses on encouraging children to pay attention to everyday videos, not just educational content. Responses and grading are also done through voice detection in real time. 
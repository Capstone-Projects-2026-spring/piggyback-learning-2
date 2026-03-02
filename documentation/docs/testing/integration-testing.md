---
sidebar_position: 2
---
# Integration tests

All integration tests are in `tests/test_integration.py` and use FastAPI's `TestClient` to simulate HTTP requests without running a real server.

## Overview
These tests verify that the API endpoints work correctly end-to-end. They are derived from the following use cases:
- **Student answers a quiz question** - tests `/api/check_answer`
- **App loads configuration for the frontend** - tests `/api/config`

## Integration Test for Use Case 1 - Admin creates quiz

An admin would like to log in and manage quizzes.

Upon opening the app, the admin enters their password.
The system verifies the password and loads the app configuration.

**Details**
- Runs `test_get_config`
- Passes if all tests pass.

## Integration Test for Use Case 2 - Learner watches video and answers quiz

A learner would like to start a quiz and answer questions.

Upon opening the app, the learner sees a list of available quizzes.
Upon selecting a quiz, the system displays the questions for that video.

**Details**
- Runs `test_learner_can_fetch_video_list`
- Runs `test_learner_can_fetch_questions_for_video`
- Passes if all tests pass.

## Integration Test for Use Case 3 - Learner answers a question using voice

A learner would like to answer a quiz question using their voice and have it scored.

The system displays a question, the learner speaks their answer.
The system converts speech to text and grades it as correct or wrong.

**Details**
- Runs `test_check_answer_correct`
- Runs `test_check_answer_wrong`
- Passes if all tests pass.

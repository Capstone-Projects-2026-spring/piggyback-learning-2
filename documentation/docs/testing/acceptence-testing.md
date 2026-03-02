---
sidebar_position: 3
---
# Acceptance test

All acceptance tests are in `tests/test_acceptance.py` and use FastAPI's `TestClient` to simulate full user scenarios.

> Note: Automated tests simulate the grading step only. Speech-to-text input is tested manually (see Manual Tests section below).

## Scenario 1: Student loads app and answers correctly
A student opens the app, the config loads, then they speak a correct answer and receive confirmation.

| Step | Action | Expected Result |
|---|---|---|
| 1 | App loads config | `status 200`, config contains `"thresholds"` |
| 2 | Student speaks correct answer, speech is transcribed and graded | `status 200`, `status: "correct"` |

## Scenario 2: Student mispronounces Pikachu but still gets it correct
A student knows the answer but mispronounces it slightly. The fuzzy matching accepts it anyway.

| Step | Action | Expected Result |
|---|---|---|
| 1 | Student speaks `"pikahu"`, speech is transcribed and fuzzy matched to  `"pikachu"` | `status 200`, `status: "correct"` |

## Scenario 3: Student answers spinning cat (O I I A I) video question
A student watches the spinning(O I I A I) cat video and mispronounces their answer. Fuzzy matching accepts it.

| Step | Action | Expected Result |
|---|---|---|
| 1 | Student speaks `"spinng"`, speech is transcribed and fuzzy matched to `"spinning"` | `status 200`, `status: "correct"` |

## Manual Tests

These scenarios require a real browser and microphone and were tested manually.

### Scenario 4: Student speaks answer into microphone
1. Open the app in a browser
2. Navigate to a video with a quiz question
3. Click the microphone button and say the answer out loud
4. Observe that the speech is transcribed into the answer field
5. Observe that the result shows "correct" or "wrong"

**Observed Result:** Passed , speech was correctly transcribed and graded.
---
sidebar_position: 5
---

# Use-case descriptions

# Use Case 1 - Admin creates quiz
*As a parent or administrator, I want to generate a quiz for the child to use.*
1. After logging in, the admin selects the 'Administrator' option on the 'Choose your role' page.
2. The app prompts the admin and the admin uploads a video.
3. The system processes the video and generates a quiz.
4. The administrator approves the quiz to be completed by the child.

# Use Case 2 - Learner watches video and answers quiz
*As a user, I want to start a quiz so I can watch a video and answer questions.*
1. After logging in, the user sees a list of available quizzes.
2. The user selects a quiz.
3. The system displays quiz instructions, including how to answer using voice.
4. The user selects “Start Quiz”.
5. The system displays the first question. 

# Use Case 3 - Learner answers a question using voice
*As a user, I want to answer quiz questions using my voice and have my answer scored moments later.*
1. The system displays a question and prompts the user to speak their answer.
2. The system automatically starts to record and the user speaks their answer.
3. The system records the audio and converts it into text.
4. The system displays the recognized answer.
5. The user confirms the answer, and the system saves it and moves to the next question.

**Alternate flow**: If the system cannot recognize the speech clearly, the user is prompted to retry speaking.


# Use Case 4 - Track progress
*As a child user, I want the option to rewind the video or continue the quiz so I can review the lesson if I am unsure about an answer.*

1. The child logs into the quiz web application and starts a quiz.
2. The system displays a quiz question related to a section of an educational video.
3. The child answers the question using voice input.
4. The system evaluates the answer and determines whether it is correct or incorrect.
5. If the answer is incorrect, the system displays two options: “Rewind Video” and “Keep Going.”
6. If the child selects “Rewind Video,” the system rewinds the video to the timestamp associated with the question and plays that section.
7. After the video segment finishes, the child can return to the quiz question or continue to the next question.

**Alternative flow**:  If the child selects “Keep Going,” the system proceeds directly to the next quiz question without replaying the video.

# Use Case 5 - Parental report
*As a parent or guardian, I want to check on my child's progress to be able to make adjustments as necessary.*
1. After logging in, the admin selects the 'Administrator' option on the 'Choose your role' page.
2. The admin clicks on the 'Dashboard' button.
3. The admin views a page containing Piggyback's results, including response scores, time watched, and other insights.
3. The admin views a page containing Piggyback's results, including response scores, time watched, and other insights.

# Use Case 6 - Expert review
*As a parent or guardian, I want to be able to review and modify the quizzes for my child.*
1. After logging in, the admin selects the 'Expert Reviewer' option on the 'Choose your role' page.
2. The screen displays the quizzes that the admin has created and approved.
3. The admin selects one of the quizzes to be reviewed.
4. The admin selects a timestamp with a question and rewinds slightly to look through the video.
5. The admin makes changes to the question as needed.
6. Once done with the whole quiz, the admin saves the quiz for the child to be able to select again.
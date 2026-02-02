---
sidebar_position: 5
---

# Use-case descriptions

# Use Case 1 - Create an account
*As a user, it is important that I can create an account so that I can start using the website and opening activities.*
1. When the user first opens Piggyback Learning, they are directed to a landing page, where there are buttons for creating an account or logging in. 
2. The user selects 'Register' to open the account creation page.
3. The user inputs a username, email address, password. The user creates a profile for their child and clicks the 'Sign Up' button to submit the form. If the information is vaild, the user is notified that their account creation was successful.
4. The user is redirected to the home page.

# Use Case 2 - Login
*As a user (child, teen, or parent/guardian), I want to log into my account so I can access the appropriate dashboard.*
1. After logging in, the user sees a list of available quizzes.
2. The user selects a quiz.
3. The system displays quiz instructions, including how to answer using voice.
4. The user selects “Start Quiz”.
5. The system displays the first question.

# Use Case 3 - Start a quiz
*As a user, I want to start a quiz so I can answer questions and learn.*
1. After logging in, the user sees a list of available quizzes.
2. The user selects a quiz.
3. The system displays quiz instructions, including how to answer using voice.
4. The user selects “Start Quiz”.
5. The system displays the first question. 


# Use Case 4 - Answer a question using voice
*As a user, I want to answer quiz questions using my voice instead of typing.*
1. The system displays a question and prompts the user to speak their answer.
2. The user clicks the microphone button and speaks their answer.
3. The system records the audio and converts it into text.
4. The system displays the recognized answer.
5. The user confirms the answer, and the system saves it and moves to the next question.

**Alternate flow**: If the system cannot recognize the speech clearly, the user is prompted to retry speaking.

# Use Case 5 - Track progress
*As a child user, I want the option to rewind the video or continue the quiz so I can review the lesson if I am unsure about an answer.*

1.The child logs into the quiz web application and starts a quiz.
2.The system displays a quiz question related to a section of an educational video.
3.The child answers the question using voice input.
4.The system evaluates the answer and determines whether it is correct or incorrect.
5.If the answer is incorrect, the system displays two options: “Rewind Video” and “Keep Going.”
6.If the child selects “Rewind Video,” the system rewinds the video to the timestamp associated with the question and plays that section.
7.After the video segment finishes, the child can return to the quiz question or continue to the next question.

*Alternative flow*:  If the child selects “Keep Going,” the system proceeds directly to the next quiz question without replaying the video.
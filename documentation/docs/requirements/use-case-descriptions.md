---
sidebar_position: 5
---

# Use-case descriptions

### Use Case 1 - Account Creation

*As a user, it is important that I can create an account so that I can maintain my video preferences and settings.* 

1.  Upon accessing the web application for the first time, the user is directed to a dashboard. There are buttons for creating an account and logging in.
2.  The user selects the 'Register' button to access the account registration form.
3.  The user inputs their username, email address, password, and other optional fields, and clicks the 'Sign Up' button to submit the form. If the information is valid, the user is notified that their account creation was successful.
4.  The user receives an email which contains a link that verifies their email account. (optional)
5.  The user selects the 'Sign In' link to access the sign-in page, and enters in their email and password.
6.  Since the user is logging in for the first time, they are redirected to a landing page (tutorial?)

---

### Use case 2 - Find other Accounts

*As a user, I want to search for other users so I can assign videos and quizzes to them.*

1.  The user opens the Find Users page/search bar.
2.  The user types a name/email and searches.
3.  The system displays matching accounts.
4.  The user selects an account to view/assign content.
    
---

### Use case 3 - View Dashboard

*As a user, I want a dashboard so I can view my history and quiz performance.*

1.  The user opens the dashboard page.
2.  The system shows watch history and quiz.
3.  The user can click a video to see more detailed stats.

---

### Use Case 4 - Answering a Quiz Question

*As a user, I want to be able to answer quiz questions with voice recognition.*

1. A quiz for the video starts and asks the user a question.
2. The user answers vocally, after seeing a visual indication that voice input is being accepted (“you can speak now!” or something like that).
3. The user's input is mapped to an answer for the quiz.
4. If incorrect, a fallback option is triggered. Potentially a multiple-choice quiz
5. If correct, the video continues playing.

---

### Use Case 5 - Store Video and Data

*As a user, I want my video and quiz activity to be saved automatically so that my progress can be tracked.*

1. The data is saved in the database without the user's input.
2. The user can later view this information on their dashboard.
3. The system automatically records video information like watch time.
4. The system stores quiz results.

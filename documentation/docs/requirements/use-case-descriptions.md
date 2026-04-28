---
sidebar_position: 5
---

# Use-case descriptions

<!-- ### Use Case 1 - Account Creation

_As a user, it is important that I can create an account so that I can maintain my video preferences and settings._

1.  Upon accessing the web application for the first time, the user is directed to a dashboard. There are buttons for creating an account and logging in.
2.  The user selects the 'Register' button to access the account registration form.
3.  The user inputs their username, email address, password, and other optional fields, and clicks the 'Sign Up' button to submit the form. If the information is valid, the user is notified that their account creation was successful.
4.  The user receives an email that contains a link that verifies their email account. (optional)
5.  The user selects the 'Sign In' link to access the sign-in page, and enters their email and password.
6.  Since the user is logging in for the first time, they are redirected to a landing page (tutorial?)

--- -->
<!-- 
## New use cases
--- -->
### Use case 1 - Add children to my account and assign videos to them
_As a parent, I want to create a children's account attached to mine, so that I can download and assign videos to them._

1. Parent opens the 'Your Kids Dashboard' and enters the kid's details.
2. Application then creates the kid's account linked to the parent's account.
3. Parent searches for a YouTube video and tells the app to download it.
4. The app downloads the video and extracts the frames.
5. Parent sets time intervals for the quiz and starts question generation (OpenAI).
6. Parent reviews the questions and assigns the video to their kid.

### Use case 2 - Detect if my child is paying attention to the video
_As a parent, I want to detect if my child is paying attention to the video, so that I can keep them on track._

1. The child starts watching an assigned video.
2. The eye tracker monitors the child's eyes (locally).
3. If the kid's eyes wander for a few seconds, the system will consider the kid distracted
4. The application automatically pauses the video and displays a prompt for the kid to focus.
5. The application sends a notification to the Parent.

### Use case 3 - Answering a Quiz Question with voice recognition
_As a child, I want to be able to answer questions based on the video, using my voice._

1. Video reaches a quiz timestamp and pauses.
2. The Mascot uses text-to-speech (TTS) to read the question aloud.
3. The child speaks their answer, which is recorded by the application.
4. Audio is analyzed by the app for correctness.
5. If correct, the Mascot gives feedback and the video resumes.
6. If incorrect, the system will perform a fallback (replay the video segment or question layering).

### Use case 4 - View Quiz Results for Kids
_As a parent, I want to view how well my kids did on the quizzes._
1. Parent navigates to the dashboard and requests data on Kid's quizzes.
2. The Application requests the history of answers for a specific child and video.
3. The history of answers from the Database.
4. The Parent sees the child's answers, quiz results, quiz answers, and detected mood.

### Use case 5 - Add tags to Kids Accounts
_As a parent, I want to add tags related to my kid's interests to their accounts so that I can see recommended videos relevant to what my kid likes._
1. Parent goes to the 'Kid Dashboard' of their kid.
2. Parent goes to the 'Tags' tab.
3. Parent adds tags to their kids.
4. Parent clicks the 'Recommended' tab.
5. The application runs a search to find videos with matching tags.


### See Sequence Diagrams Tab for sequence diagrams representing all of the use cases

<!--
## Below is old use cases
### Use case 1 - Find other Accounts

_As a parent, I want to search for my children so I can assign videos and quizzes to them._

1.  The user opens the Find Users page/search bar.
2.  The user types a name/email and searches.
3.  The system displays matching accounts.
4.  The user selects an account to view/assign content.

[![](https://mermaid.ink/img/pako:eNp1kk1Pg0AQhv_KZs60llKg7KGJFb0Zo40xMVzWZYSNsOB-RGvT_-4utaRp6o2Zed9nZpbZAe9KBAoaPy1KjrlglWJtIQlh3HSKPGtUPuqZMoKLnklDXvDtuu_Ps9fW1Oe5fF1In_OUyWp1MFLy0KMkd0KWQ0E7S4VedqhPnNLnKdnU3RfRyBSviWFvF1m30qAikrV4hS0TDWEOe_CcIFcrP54j4lglyu-sjVf5otPka0oeLaotaZnhtZAVsX5AL8nXk5HyhMYq6QDaNkaPgJOphj4jhHHeWemVl7bMhe4btj3Fne24wQa5OWL-fam_OgRQKVECNcpiAC0q9y4uhJ13FmBqbLEA6j5Lpj4KKOTeedwPe-269mhTna1qoO-s0S6yfcnM8TrGrHJboroZmtJwnsQDBegOvoGm8XSRZnE6S5JllIZhGMAW6DKeJlG4SNJsvoyzZB7vA_gZ2s6mSRpHUZZkyzQL41mUBYClcEd4f7jR4VT3vzJb5C8?type=png)](https://mermaid.live/edit#pako:eNp1kk1Pg0AQhv_KZs60llKg7KGJFb0Zo40xMVzWZYSNsOB-RGvT_-4utaRp6o2Zed9nZpbZAe9KBAoaPy1KjrlglWJtIQlh3HSKPGtUPuqZMoKLnklDXvDtuu_Ps9fW1Oe5fF1In_OUyWp1MFLy0KMkd0KWQ0E7S4VedqhPnNLnKdnU3RfRyBSviWFvF1m30qAikrV4hS0TDWEOe_CcIFcrP54j4lglyu-sjVf5otPka0oeLaotaZnhtZAVsX5AL8nXk5HyhMYq6QDaNkaPgJOphj4jhHHeWemVl7bMhe4btj3Fne24wQa5OWL-fam_OgRQKVECNcpiAC0q9y4uhJ13FmBqbLEA6j5Lpj4KKOTeedwPe-269mhTna1qoO-s0S6yfcnM8TrGrHJboroZmtJwnsQDBegOvoGm8XSRZnE6S5JllIZhGMAW6DKeJlG4SNJsvoyzZB7vA_gZ2s6mSRpHUZZkyzQL41mUBYClcEd4f7jR4VT3vzJb5C8)

---

<!-- ### Use case 3 - View Dashboard

_As a user, I want a dashboard so I can view my history and quiz performance._

1.  The user opens the dashboard page.
2.  The system shows watch history and quiz.
3.  The user can click a video to see more detailed stats.

--- -->
<!--
### Use Case 2 - Answering a Quiz Question

_As a user, I want to be able to answer quiz questions with voice recognition._

1. A quiz for the video starts and asks the user a question.
2. The user answers vocally, after seeing a visual indication that voice input is being accepted (“you can speak now!” or something like that).
3. The user's input is mapped to an answer for the quiz.
4. If incorrect, a fallback option is triggered. Potentially a multiple-choice quiz
5. If correct, the video continues playing.

[![](https://mermaid.ink/img/pako:eNqVkzFv2zAQhf_K9WbZsKxIsjgYsOMlQ4G0RRGg0MJIF4uIRKokZdcx_N9LirHjNs1QTeLx43vHR_KIlaoJGRr6OZCsaCP4VvOulOA-Xlml4bshHcY911ZUoufSwgM9rvr-fX012OZ9dbMuZah6tclyGZYzeOC2asjATtSkAhGmJg7yLIMvg3iBXquut7AXtnGsGXgLQtai4lYo-YH2Spo9aQNuZ8ZjsCP9yNv28IfPculbZvCNZA07JSoCTZXStZDbAPp5h23WvhfSB-h8024a-OgQqM16ctH6SnbQ0umYobXmSuWqu9EvEKCeLlKvybuiMODacL3YUPtXNhth-pYfoMTbgH4qEd7wvwK516oiqkOK-_MmrqKn1pD3FfI_nO8uMH6M34fzuxwF33IhQyOdS0D0LUHV-PRf43LpYIRbLWpkVg8UYUe6436IR4-UaBvqqETmfmuun739ya1xF-6HUt15mVbDtkH2xN3eIhz6mtvzLb9UtXMjfasGaZHF8zQZVZAd8ReyPJ3e5EWaz7JskeRxHEd4QLZIp1kS32R5MV-kRTZPTxG-jLazaZanSVJkxSIv4nSWFBFSLdxT-hze2vjkTr8BZWggRw?type=png)](https://mermaid.live/edit#pako:eNqVkzFv2zAQhf_K9WbZsKxIsjgYsOMlQ4G0RRGg0MJIF4uIRKokZdcx_N9LirHjNs1QTeLx43vHR_KIlaoJGRr6OZCsaCP4VvOulOA-Xlml4bshHcY911ZUoufSwgM9rvr-fX012OZ9dbMuZah6tclyGZYzeOC2asjATtSkAhGmJg7yLIMvg3iBXquut7AXtnGsGXgLQtai4lYo-YH2Spo9aQNuZ8ZjsCP9yNv28IfPculbZvCNZA07JSoCTZXStZDbAPp5h23WvhfSB-h8024a-OgQqM16ctH6SnbQ0umYobXmSuWqu9EvEKCeLlKvybuiMODacL3YUPtXNhth-pYfoMTbgH4qEd7wvwK516oiqkOK-_MmrqKn1pD3FfI_nO8uMH6M34fzuxwF33IhQyOdS0D0LUHV-PRf43LpYIRbLWpkVg8UYUe6436IR4-UaBvqqETmfmuun739ya1xF-6HUt15mVbDtkH2xN3eIhz6mtvzLb9UtXMjfasGaZHF8zQZVZAd8ReyPJ3e5EWaz7JskeRxHEd4QLZIp1kS32R5MV-kRTZPTxG-jLazaZanSVJkxSIv4nSWFBFSLdxT-hze2vjkTr8BZWggRw)

---

### Use Case 3 - Store Video and Data

_As a user, I want my video and quiz activity to be saved automatically so that my progress can be tracked._

1. The data is saved in the database without the user's input.
2. The user can later view this information on their dashboard.
3. The system automatically records video information like watch time.
4. The system stores quiz results.

[![](https://mermaid.ink/img/pako:eNptkk9PwkAQxb_KZs6IlNqW7sFEbLwZE40xMb2M3RE20N26f1AkfHd3KRhAL83O5L23v53OBhotCDhY-vCkGqokzgy2tWIMG6cNe7ZkYtWhcbKRHSrHXujtpuvOu1NsFqTEebua1ir2YtDF9XXv5ewFXTNnKylIs0vmcEHsw8vvqOwlQbtP5OwpfFln9MyQtZdRF-nkSrp1NOx1wVFNgxhXxD538U621MuD0S-djepqenGSHeTiX8SHjhSr0M7fNBrBluj6Ufzle4zTs47ZmPUL-hftjiKVD9ec8J8RPZLzRjGBDo8jjshO5nGEFCTxDZxV0nZLXB-eDQOYGSmAO-NpAC2ZFmMJm2iuwc2ppRp4OAo0ixpqtQ2e8P9etW4PNqP9bA78HZc2VL4LgId9-e2aQEbmVnvlgCfjvNylAN_AF_AiG14VZVaM8nySFkmSDGANfJIN8zS5yotyPMnKfJxtB_C9u3Y0zIssTcu8nBRlko3SkEZChrW877d2t7zbH1GT6rk?type=png)](https://mermaid.live/edit#pako:eNptkk9PwkAQxb_KZs6IlNqW7sFEbLwZE40xMb2M3RE20N26f1AkfHd3KRhAL83O5L23v53OBhotCDhY-vCkGqokzgy2tWIMG6cNe7ZkYtWhcbKRHSrHXujtpuvOu1NsFqTEebua1ir2YtDF9XXv5ewFXTNnKylIs0vmcEHsw8vvqOwlQbtP5OwpfFln9MyQtZdRF-nkSrp1NOx1wVFNgxhXxD538U621MuD0S-djepqenGSHeTiX8SHjhSr0M7fNBrBluj6Ufzle4zTs47ZmPUL-hftjiKVD9ec8J8RPZLzRjGBDo8jjshO5nGEFCTxDZxV0nZLXB-eDQOYGSmAO-NpAC2ZFmMJm2iuwc2ppRp4OAo0ixpqtQ2e8P9etW4PNqP9bA78HZc2VL4LgId9-e2aQEbmVnvlgCfjvNylAN_AF_AiG14VZVaM8nySFkmSDGANfJIN8zS5yotyPMnKfJxtB_C9u3Y0zIssTcu8nBRlko3SkEZChrW877d2t7zbH1GT6rk)

--- -->

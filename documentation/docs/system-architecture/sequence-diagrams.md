---
sidebar_position: 6
---

# Sequence Diagrams

## Use case 1 - Admin creates quiz

```mermaid
sequenceDiagram
    participant Admin
    participant App
    participant System

    Admin->>App: Log in
    App->>Admin: Display 'Choose your role'
    Admin->>App: Select 'Administrator'
    App->>Admin: Prompt to upload video
    Admin->>App: Upload video
    App->>System: Process video & generate quiz
    System-->>App: Quiz generated
    Admin->>App: Approve quiz
    App-->>System: Quiz ready for child
```

## Use case 2 - Learner watches the video and answers quiz

```mermaid
sequenceDiagram
    participant User
    participant App
    participant System

    User->>App: Log in
    App->>User: Display available quizzes
    User->>App: Select a quiz
    App->>User: Display quiz instructions
    User->>App: Start Quiz
    App->>System: Load first question
    System-->>App: Display question
```

## Use case 3 - Learner answers a question using voice

```mermaid
sequenceDiagram
    participant User
    participant App
    participant System
    participant Speech

    App->>User: Display question, prompt to speak
    User->>App: Speak answer
    App->>Speech: Record audio
    Speech-->>App: Convert speech to text
    App->>User: Show recognized answer
    User->>App: Confirm answer
    App->>System: Save answer, move to next question
    alt Speech not recognized
        App->>User: Prompt to retry speaking
    end
```

## Use case 4 - Track progress

```mermaid
sequenceDiagram
    participant Child
    participant App
    participant System
    participant Video

    Child->>App: Log in & start quiz
    App->>System: Display question
    Child->>App: Answer using voice
    App->>System: Evaluate answer
    alt Answer correct
        System-->>App: Move to next question
    else Answer incorrect
        App->>Child: Show options "Rewind Video" or "Keep Going"
        alt Rewind Video
            App->>Video: Rewind to question timestamp
            Video-->>Child: Play segment
            Child->>App: Return to question or continue
        else Keep Going
            System-->>App: Move to next question
        end
    end
```

## Use case 5 - Parental report

```mermaid
sequenceDiagram
    participant Admin
    participant App
    participant System

    Admin->>App: Log in
    App->>Admin: Display 'Choose your role'
    Admin->>App: Select 'Administrator'
    Admin->>App: Click 'Dashboard'
    App->>System: Fetch child results
    System-->>App: Send scores, time watched, insights
    App-->>Admin: Display report
```

## Use case 6 - Expert review

```mermaid
sequenceDiagram
    participant Admin
    participant App
    participant System
    participant Video

    Admin->>App: Log in
    App->>Admin: Display 'Choose your role'
    Admin->>App: Select 'Expert Reviewer'
    App->>Admin: Show created/approved quizzes
    Admin->>App: Select quiz to review
    App->>Video: Rewind to timestamp for question
    Admin->>App: Modify question as needed
    App->>System: Save updated quiz
    System-->>App: Quiz ready for child selection
```

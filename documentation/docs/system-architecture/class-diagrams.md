---
sidebar_position: 5
---

# Class Diagrams

## Backend classes

```mermaid
classDiagram
    class QuizManager {
        -quizzes: List~Quiz~
        -storage: StorageManager
        -videoProcessor: VideoProcessor
        -questionGenerator: AIQuestionGenerator
        -ttsModule: TTSModule

        +createQuiz(videoUrl)
        +approveQuiz(quizId)
        +getQuiz(quizId)
        +evaluateAnswer(answer)
        +saveQuiz(quiz)
    }

    class Quiz {
        -id: string
        -title: string
        -questions: List~Question~
        -approved: boolean

        +addQuestion(question)
        +setApproved(status)
    }

    class Question {
        -text: string
        -options: List~string~
        -timestamp: float
        -correctAnswer: string

        +validateAnswer(answer)
    }

    class Answer {
        -response: string
        -timestamp: datetime
        -score: int

        +calculateScore(correctAnswer)
    }

    class VideoProcessor {
        -videoPath: string
        -frameDirectory: string

        +downloadVideo(url)
        +extractFrames()
        +extractTranscript()
    }

    class AIQuestionGenerator {
        -modelName: string
        -openAIService: OpenAIService

        +generateQuestions(transcript)
        +validateQuestions(questions)
    }

    class TTSModule {
        -voiceModel: string

        +convertTextToSpeech(text)
    }

    QuizManager --> Quiz
    Quiz --> Question
    QuizManager --> VideoProcessor
    QuizManager --> AIQuestionGenerator
    QuizManager --> TTSModule
    QuizManager --> StorageManager
    Question --> Answer
```
The backend component manages quiz creation, video processing, 
AI-based question generation, and evaluation of user responses. The QuizManager
acts as the central coordinator and interacts with the VideoProcessor,
AIQuestionGenerator, TTSModule, and StorageManager. The design helps with 
modular processing and responsibility separation.


## Frontend classes

```mermaid
classDiagram
    class KidsUI {
        -currentQuiz: Quiz

        +displayQuiz(quiz)
        +playVideo(timestamp)
        +submitAnswer(answer)
        +rewindVideo(timestamp)
    }

    class AdminUI {
        +uploadVideo(video)
        +approveQuiz(quizId)
        +viewDashboard()
    }

    class ExpertUI {
        +reviewQuiz(quiz)
        +editQuestion(question)
        +saveQuiz(quiz)
    }

    KidsUI --> QuizManager
    AdminUI --> QuizManager
    ExpertUI --> QuizManager
```
The frontend component provides user interfaces for children, administrators,
and expert reviewers. Each UI class communicates with the backend through
well-defined APIs. Business logic is handled by the backend, which reduces
coupling and improves maintainability.


## Video/Audio pipeline classes

```mermaid
classDiagram
    class AudioRecorder {
        -audioFilePath: string

        +startRecording()
        +stopRecording()
        +saveAudio()
    }

    class SpeechRecognizer {
        -openAIService: OpenAIService

        +convertAudioToText(audioFile)
        +retryRecognition()
    }

    AudioRecorder --> SpeechRecognizer
    SpeechRecognizer --> QuizManager
```
The video and audio processing subsystem enables voice-based interaction.
AudioRecorder captures user speech, while SpeechRecognizer converts audio
to text using external AI services. Recognized answers are then forwarded to the
backend for evaluation.


## Storage and external services

```mermaid
classDiagram
    class StorageManager {
        -videoDirectory: string
        -frameDirectory: string
        -quizDataStore: string

        +saveVideo(video)
        +saveFrames(frames)
        +saveQuizData(quiz)
        +loadQuizData(quizId)
    }

    class OpenAIService {
        -apiKey: string

        +generateQuestions(prompt)
        +speechToText(audio)
    }

    class YouTubeService {
        +downloadVideo(url)
    }

    class FFmpegService {
        +processVideo(video)
    }

    StorageManager --> QuizManager
    OpenAIService --> AIQuestionGenerator
    OpenAIService --> SpeechRecognizer
    YouTubeService --> VideoProcessor
    FFmpegService --> VideoProcessor
```
The storage component manages persistent data including videos, frames,
and quiz metadata. External service wrappers encapsulate third-party
integrations like OpenAI, YouTube downloading, and FFmpeg processing,
which reduces direct dependencies within the core application.

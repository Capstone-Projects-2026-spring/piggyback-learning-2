import { useState, useRef, useCallback, useEffect } from "react";
import { useSegments } from "../hooks/useSegments.js";
import { useAudioRecorder } from "../hooks/useAudioRecorder.js";
import { usePlaybackPoller } from "../hooks/usePlaybackPoller.js";
import { useGazeTracker } from "../hooks/useGazeTracker.js";
import { useLocalVideo } from "../hooks/useLocalVideo.js";
import { invoke } from "@tauri-apps/api/core";

const pauseMessages = [
  "Let's go — what happened? ▶️",
  "Why did we stop? Let's keep watching 👀",
  "Come on, we were doing so good 😄",
];

export default function WatchVideoPanel({ videoId, onClose }) {
  const [currentQuestion, setCurrentQuestion] = useState(null);
  const [isFollowup, setIsFollowup] = useState(false);
  const [followupType, setFollowupType] = useState(null);
  const [segmentIndex, setSegmentIndex] = useState(0);
  const [recordingState, setRecordingState] = useState("idle");
  const [statusMessage, setStatusMessage] = useState("");
  const [analysisResult, setAnalysisResult] = useState(null);
  const [lookingAway, setLookingAway] = useState(false);
  const [piggyMode, setPiggyMode] = useState("watch");
  const [piggyText, setPiggyText] = useState("Let's watch carefully 👀");

  const playerRef = useRef(null);
  const segmentIndexRef = useRef(0);
  const currentQuestionRef = useRef(null);
  const sixSecondShownRef = useRef(false);
  const threeSecondShownRef = useRef(false);

  const { segmentsRef, videoPath } = useSegments(videoId);
  const { blobUrl, loading: videoLoading } = useLocalVideo(videoPath);

  useEffect(() => {
    invoke("gaze_start").catch(() => {});
    return () => {
      invoke("gaze_stop").catch(() => {});
    };
  }, []);

  useEffect(() => {
    if (!blobUrl || !playerRef.current) return;
    playerRef.current.load();
    playerRef.current.play().catch((e) => {
      console.error("[WatchVideoPanel] autoplay failed:", e);
    });
  }, [blobUrl]);

  useEffect(() => {
    segmentIndexRef.current = segmentIndex;
    sixSecondShownRef.current = false;
    threeSecondShownRef.current = false;
    setPiggyMode("watch");
    setPiggyText("Let's watch carefully 👀");
  }, [segmentIndex]);

  useEffect(() => {
    currentQuestionRef.current = currentQuestion;
  }, [currentQuestion]);

  useEffect(() => {
    if (currentQuestion) setLookingAway(false);
  }, [currentQuestion?.question]);

  const advanceAndPlay = useCallback(() => {
    setCurrentQuestion(null);
    setAnalysisResult(null);
    setRecordingState("idle");
    setStatusMessage("");
    setSegmentIndex((prev) => prev + 1);
    setPiggyMode("talk");
    setPiggyText("Nice! Let's keep watching 🎬");
    setTimeout(() => {
      setPiggyMode("watch");
      setPiggyText("Let's watch carefully 👀");
    }, 2000);
    playerRef.current?.play();
  }, []);

  const replaySegment = useCallback(() => {
    const segs = segmentsRef.current;
    const idx = segmentIndexRef.current;
    if (!playerRef.current || idx >= segs.length) return;

    sixSecondShownRef.current = false;
    threeSecondShownRef.current = false;

    setPiggyMode("talk");
    setPiggyText("Let's try that part again 👀");
    setTimeout(() => {
      setPiggyMode("watch");
      setPiggyText("Let's watch carefully 👀");
    }, 2000);

    playerRef.current.currentTime = segs[idx].start_seconds ?? 0;
    playerRef.current.play();
  }, [segmentsRef]);

  const handleFollowupResult = useCallback(
    (result) => {
      if (!result) {
        advanceAndPlay();
        return;
      }
      setAnalysisResult(result);
      if (result.is_correct) {
        setRecordingState("correct");
        setStatusMessage("Correct! Well done 🎉");
        setTimeout(() => advanceAndPlay(), 2000);
      } else {
        setRecordingState("wrong");
        setStatusMessage("Not quite — let's rewatch!");
        setTimeout(() => {
          setIsFollowup(false);
          setFollowupType(null);
          setCurrentQuestion(null);
          setAnalysisResult(null);
          setTimeout(() => replaySegment(), 100);
        }, 2000);
      }
    },
    [advanceAndPlay, replaySegment],
  );

  const handleResult = useCallback(
    (result) => {
      if (!result) {
        advanceAndPlay();
        return;
      }
      const q = currentQuestionRef.current;
      setAnalysisResult(result);

      if (result.is_correct) {
        setRecordingState("correct");
        if (q?.followup_correct_question) {
          setStatusMessage("Correct! Let's try a quick follow-up.");
          setTimeout(() => {
            setAnalysisResult(null);
            setRecordingState("idle");
            setStatusMessage("");
            setFollowupType("correct");
            setIsFollowup(true);
          }, 2000);
        } else {
          setStatusMessage("Correct! Well done 🎉");
          setTimeout(() => advanceAndPlay(), 2000);
        }
      } else {
        setRecordingState("wrong");
        if (q?.followup_wrong_question) {
          setStatusMessage("Not quite — try answering this instead!");
          setTimeout(() => {
            setAnalysisResult(null);
            setRecordingState("idle");
            setStatusMessage("");
            setFollowupType("wrong");
            setIsFollowup(true);
          }, 2000);
        } else {
          setStatusMessage("Not quite — let's rewatch!");
          setTimeout(() => {
            setCurrentQuestion(null);
            setAnalysisResult(null);
            setTimeout(() => replaySegment(), 100);
          }, 2000);
        }
      }
    },
    [advanceAndPlay, replaySegment],
  );

  const recorder = useAudioRecorder({
    onStateChange: setRecordingState,
    onStatusChange: setStatusMessage,
    onResult: isFollowup ? handleFollowupResult : handleResult,
  });

  usePlaybackPoller({
    playerRef,
    segmentsRef,
    segmentIndexRef,
    currentQuestionRef,
    onTick: (currentTime, segment) => {
      if (!segment || currentQuestionRef.current) return;
      const end = segment.end_seconds ?? 0;
      const timeLeft = end - currentTime;

      if (timeLeft <= 6 && timeLeft > 3 && !sixSecondShownRef.current) {
        sixSecondShownRef.current = true;
        setPiggyMode("talk");
        setPiggyText("Pay attention — a question is coming 👀");
      }
      if (timeLeft <= 3 && timeLeft > 0 && !threeSecondShownRef.current) {
        threeSecondShownRef.current = true;
        setPiggyMode("talk");
        setPiggyText("Get ready to answer! 🎤");
      }
    },
    onSegmentEnd: (segment) => {
      const match = segment.questions?.find(
        (x) =>
          x.question === segment.best_question ||
          x.qtype === segment.best_question ||
          x.question?.includes(segment.best_question) ||
          (segment.best_question && segment.best_question.includes(x.question)),
      );
      setCurrentQuestion(
        match || {
          question: segment.best_question || "What did you just see?",
        },
      );
    },
  });

  useGazeTracker({
    enabled: true,
    paused: !!currentQuestion,
    onLookAway: () => {
      playerRef.current?.pause();
      setLookingAway(true);
    },
    onReturn: () => {
      setLookingAway(false);
      playerRef.current?.play();
    },
  });

  useEffect(() => {
    if (!currentQuestion) return;
    const segment = segmentsRef.current[segmentIndexRef.current];
    if (!segment) return;

    const questionToAsk = isFollowup
      ? {
          question:
            followupType === "correct"
              ? currentQuestion.followup_correct_question
              : currentQuestion.followup_wrong_question,
          answer:
            followupType === "correct"
              ? currentQuestion.followup_correct_answer
              : currentQuestion.followup_wrong_answer,
        }
      : segment;

    recorder.start(questionToAsk, {
      kid_id: 0,
      video_id: videoId,
      segment_id: segment.id,
      ...(isFollowup && { expected_answer_override: questionToAsk?.answer }),
    });

    return () => recorder.cancel();
  }, [currentQuestion, isFollowup]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleCloseQuestion = useCallback(() => {
    recorder.cancel();
    setRecordingState("idle");
    setStatusMessage("");
    setAnalysisResult(null);
    advanceAndPlay();
  }, [recorder, advanceAndPlay]);

  const displayQuestion = isFollowup
    ? followupType === "correct"
      ? currentQuestion?.followup_correct_question
      : currentQuestion?.followup_wrong_question
    : currentQuestion?.question;

  if (!blobUrl || videoLoading) {
    return (
      <div className="fixed inset-0 z-50 bg-black flex flex-col items-center justify-center gap-3">
        <div className="w-8 h-8 rounded-full border-2 border-white/20 border-t-white/60 animate-spin" />
        <p className="text-white/40 text-xs">Loading video…</p>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 z-50 bg-black flex flex-col">
      {/* Top bar */}
      <div className="absolute top-0 left-0 right-0 z-10 flex items-center justify-between px-4 py-3 bg-gradient-to-b from-black/70 to-transparent">
        <button
          onClick={onClose}
          className="w-8 h-8 rounded-full bg-white/10 flex items-center justify-center text-white/60 hover:text-white hover:bg-white/20 transition-colors"
        >
          <svg
            className="w-4 h-4"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15 19l-7-7 7-7"
            />
          </svg>
        </button>

        {recordingState !== "idle" && (
          <div
            className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium border ${
              recordingState === "recording"
                ? "bg-red-500/20 text-red-300 border-red-500/30"
                : recordingState === "correct"
                  ? "bg-green-500/20 text-green-300 border-green-500/30"
                  : recordingState === "wrong"
                    ? "bg-red-500/20 text-red-300 border-red-500/30"
                    : "bg-white/10 text-white/60 border-white/10"
            }`}
          >
            {recordingState === "recording" && (
              <span className="w-2 h-2 rounded-full bg-red-400 animate-pulse" />
            )}
            {statusMessage}
          </div>
        )}

        <div className="w-8" />
      </div>

      {/* Video */}
      <video
        ref={playerRef}
        src={blobUrl}
        className="w-full h-full object-contain"
        onPause={() => {
          if (currentQuestionRef.current || lookingAway) return;
          setPiggyMode("talk");
          setPiggyText(
            pauseMessages[Math.floor(Math.random() * pauseMessages.length)],
          );
        }}
        onPlay={() => {
          setPiggyMode("watch");
          setPiggyText("Let's watch carefully 👀");
        }}
        onEnded={onClose}
        onError={(e) => {
          console.error(
            "[video] error:",
            e.target.error?.code,
            e.target.error?.message,
          );
        }}
      />

      {!displayQuestion && !lookingAway && (
        <PiggyCompanion mode={piggyMode} text={piggyText} />
      )}

      {lookingAway && !currentQuestion && <LookAtScreenModal />}

      {displayQuestion && (
        <QuestionOverlay
          question={displayQuestion}
          recordingState={recordingState}
          statusMessage={statusMessage}
          analysisResult={analysisResult}
          isFollowup={isFollowup}
          onClose={handleCloseQuestion}
        />
      )}
    </div>
  );
}

function PiggyCompanion({ mode, text }) {
  if (mode === "hidden") return null;
  return (
    <div className="absolute bottom-6 right-6 flex flex-col items-end gap-2 pointer-events-none z-10">
      {mode === "talk" && text && (
        <div className="max-w-xs bg-white/90 backdrop-blur-sm rounded-2xl rounded-br-sm px-3 py-2 shadow-lg">
          <p className="text-xs text-gray-700 font-medium leading-snug">
            {text}
          </p>
        </div>
      )}
      <div className="w-14 h-14 rounded-full bg-pink-100 border-2 border-pink-200 flex items-center justify-center text-2xl shadow-lg">
        🐷
      </div>
    </div>
  );
}

function LookAtScreenModal() {
  return (
    <div className="absolute inset-0 bg-black/80 backdrop-blur-sm flex flex-col items-center justify-center gap-4 z-20">
      <div className="text-5xl animate-bounce">👀</div>
      <p className="text-white text-lg font-semibold">
        Hey, look at the screen!
      </p>
      <p className="text-white/50 text-sm">
        The video will resume when you're back
      </p>
    </div>
  );
}

function QuestionOverlay({
  question,
  recordingState,
  statusMessage,
  analysisResult,
  isFollowup,
  onClose,
}) {
  return (
    <div className="absolute inset-0 bg-black/70 backdrop-blur-sm flex items-end justify-center pb-8 px-5 z-20">
      <div className="w-full max-w-lg bg-white rounded-3xl overflow-hidden shadow-2xl">
        <div className="bg-gradient-to-r from-pink-400 to-violet-400 px-5 py-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="text-xl">🎤</span>
            <span className="text-white font-semibold text-sm">
              {isFollowup ? "Follow-up Question" : "Time to answer!"}
            </span>
          </div>
          <button
            onClick={onClose}
            className="text-white/70 hover:text-white text-sm font-medium transition-colors"
          >
            Skip ›
          </button>
        </div>

        <div className="p-5 flex flex-col gap-4">
          <p className="text-gray-800 font-semibold text-base leading-snug">
            {question}
          </p>

          {recordingState === "waiting" && (
            <div className="flex items-center gap-2 text-amber-500 text-sm">
              <span className="w-2 h-2 rounded-full bg-amber-400 animate-pulse" />
              {statusMessage}
            </div>
          )}
          {recordingState === "recording" && (
            <div className="flex items-center gap-2 text-red-500 text-sm">
              <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
              {statusMessage}
            </div>
          )}
          {recordingState === "analyzing" && (
            <div className="flex items-center gap-2 text-violet-500 text-sm">
              <div className="w-4 h-4 border-2 border-violet-300 border-t-violet-500 rounded-full animate-spin" />
              {statusMessage}
            </div>
          )}
          {(recordingState === "correct" || recordingState === "wrong") && (
            <div
              className={`flex items-center gap-2 rounded-xl px-3 py-2.5 text-sm font-medium ${
                recordingState === "correct"
                  ? "bg-green-50 border border-green-100 text-green-600"
                  : "bg-red-50 border border-red-100 text-red-500"
              }`}
            >
              <span>{recordingState === "correct" ? "✓" : "✗"}</span>
              {statusMessage}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

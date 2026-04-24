import { useState, useRef, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useSegments } from "../hooks/useSegments.js";
import { useAudioRecorder } from "../hooks/useAudioRecorder.js";
import { useGazeTracker } from "../hooks/useGazeTracker.js";

const pauseMessages = [
  "Let's go — what happened? ▶️",
  "Why did we stop? Let's keep watching 👀",
  "Come on, we were doing so good 😄",
];

export default function WatchVideoPanel({ videoId, onClose }) {
  const [currentQuestion, setCurrentQuestion] = useState(null);
  const [isFollowup, setIsFollowup] = useState(false);
  const [followupType, setFollowupType] = useState(null);
  const [recordingState, setRecordingState] = useState("idle");
  const [statusMessage, setStatusMessage] = useState("");
  const [analysisResult, setAnalysisResult] = useState(null);
  const [lookingAway, setLookingAway] = useState(false);
  const [piggyMode, setPiggyMode] = useState("loading");
  const [piggyText, setPiggyText] = useState("Loading video…");
  const [launched, setLaunched] = useState(false);

  const currentQuestionRef = useRef(null);
  const segmentIndexRef = useRef(0);
  const sixSecondShownRef = useRef(false);
  const threeSecondShownRef = useRef(false);

  const { segments, segmentsRef, videoPath } = useSegments(videoId);

  // Launch mpv once we have segments + path
  useEffect(() => {
    if (!videoPath || segments.length === 0 || launched) return;
    setLaunched(true);

    const segmentInfos = segments.map((s) => ({
      id: s.id,
      start_seconds: s.start_seconds,
      end_seconds: s.end_seconds,
    }));

    invoke("launch_video", { path: videoPath, segments: segmentInfos })
      .then(() => {
        eprintln("[WatchVideoPanel] mpv launched");
        setPiggyMode("watch");
        setPiggyText("Let's watch carefully 👀");
      })
      .catch((e) => {
        console.error("[WatchVideoPanel] launch failed:", e);
        setPiggyText("Failed to launch video");
      });
  }, [videoPath, segments, launched]);

  // Gaze tracking
  useEffect(() => {
    invoke("gaze_start").catch(() => {});
    return () => {
      invoke("gaze_stop").catch(() => {});
      invoke("mpv_quit").catch(() => {});
    };
  }, []);

  useEffect(() => {
    currentQuestionRef.current = currentQuestion;
  }, [currentQuestion]);

  useEffect(() => {
    if (currentQuestion) setLookingAway(false);
  }, [currentQuestion?.question]);

  // mpv position tick — drive piggy countdown
  useEffect(() => {
    let unlisten;
    listen("peppa://mpv-tick", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      const pos = data.position;
      const segs = segmentsRef.current;
      const idx = segmentIndexRef.current;
      if (!segs[idx] || currentQuestionRef.current) return;

      const end = segs[idx].end_seconds;
      const timeLeft = end - pos;

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
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [segmentsRef]);

  // Segment end — show question
  useEffect(() => {
    let unlisten;
    listen("peppa://segment-end", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      const segs = segmentsRef.current;
      const idx = segmentIndexRef.current;
      const segment = segs[idx];
      if (!segment) return;

      sixSecondShownRef.current = false;
      threeSecondShownRef.current = false;

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
      setPiggyMode("hidden");
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [segmentsRef]);

  const advanceAndPlay = useCallback(() => {
    const segs = segmentsRef.current;
    const nextIdx = segmentIndexRef.current + 1;
    segmentIndexRef.current = nextIdx;

    setCurrentQuestion(null);
    setAnalysisResult(null);
    setRecordingState("idle");
    setStatusMessage("");
    sixSecondShownRef.current = false;
    threeSecondShownRef.current = false;

    if (nextIdx >= segs.length) {
      invoke("mpv_quit").catch(() => {});
      onClose();
      return;
    }

    setPiggyMode("talk");
    setPiggyText("Nice! Let's keep watching 🎬");
    setTimeout(() => {
      setPiggyMode("watch");
      setPiggyText("Let's watch carefully 👀");
    }, 2000);

    invoke("mpv_play").catch(console.error); // restore + release already inside mpv_play
  }, [segmentsRef, onClose]);

  const replaySegment = useCallback(() => {
    const segs = segmentsRef.current;
    const idx = segmentIndexRef.current;
    if (!segs[idx]) return;

    sixSecondShownRef.current = false;
    threeSecondShownRef.current = false;

    setPiggyMode("talk");
    setPiggyText("Let's try that part again 👀");
    setTimeout(() => {
      setPiggyMode("watch");
      setPiggyText("Let's watch carefully 👀");
    }, 2000);

    invoke("mpv_seek", { seconds: segs[idx].start_seconds }).catch(
      console.error,
    );
    invoke("mpv_play").catch(console.error);
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

  useGazeTracker({
    enabled: true,
    paused: !!currentQuestion,
    onLookAway: () => {
      invoke("mpv_pause").catch(() => {});
      invoke("mpv_minimize").catch(() => {}); // ← add
      setLookingAway(true);
    },
    onReturn: () => {
      setLookingAway(false);
      if (!currentQuestionRef.current) {
        invoke("mpv_play").catch(() => {}); // already calls restore() + release_tauri_front()
      }
    },
  });

  // Start recording when question appears
  useEffect(() => {
    if (!currentQuestion) return;
    const segs = segmentsRef.current;
    const segment = segs[segmentIndexRef.current];
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

  // Transparent fullscreen overlay — mpv renders behind this
  return (
    <div className="fixed inset-0 z-50 pointer-events-none">
      {/* Only interactive elements get pointer-events back */}

      {/* Close button — always visible */}
      <div className="absolute top-0 left-0 right-0 z-10 flex items-center justify-between px-4 py-3 pointer-events-auto">
        <button
          onClick={() => {
            invoke("mpv_quit").catch(() => {});
            onClose();
          }}
          className="w-8 h-8 rounded-full bg-black/40 backdrop-blur-sm flex items-center justify-center text-white/70 hover:text-white hover:bg-black/60 transition-colors"
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
            className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium border backdrop-blur-sm ${
              recordingState === "recording"
                ? "bg-red-500/20 text-red-300 border-red-500/30"
                : recordingState === "correct"
                  ? "bg-green-500/20 text-green-300 border-green-500/30"
                  : recordingState === "wrong"
                    ? "bg-red-500/20 text-red-300 border-red-500/30"
                    : "bg-black/30 text-white/60 border-white/10"
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

      {/* Piggy */}
      {!displayQuestion && !lookingAway && (
        <PiggyCompanion mode={piggyMode} text={piggyText} />
      )}

      {/* Look away — needs pointer-events to block interaction */}
      {lookingAway && !currentQuestion && (
        <div className="pointer-events-auto">
          <LookAtScreenModal />
        </div>
      )}

      {/* Question overlay */}
      {displayQuestion && (
        <div className="pointer-events-auto">
          <QuestionOverlay
            question={displayQuestion}
            recordingState={recordingState}
            statusMessage={statusMessage}
            analysisResult={analysisResult}
            isFollowup={isFollowup}
            onClose={handleCloseQuestion}
          />
        </div>
      )}
    </div>
  );
}

function PiggyCompanion({ mode, text }) {
  if (mode === "hidden" || mode === "loading") return null;
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

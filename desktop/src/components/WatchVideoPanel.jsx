import { useState, useRef, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useSegments } from "../hooks/useSegments.js";
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
  // recordingState: "idle" | "listening" | "analyzing" | "correct" | "wrong"
  const [recordingState, setRecordingState] = useState("idle");
  const [statusMessage, setStatusMessage] = useState("");
  const [lookingAway, setLookingAway] = useState(false);
  const [piggyMode, setPiggyMode] = useState("loading");
  const [piggyText, setPiggyText] = useState("Loading video…");
  const [launched, setLaunched] = useState(false);

  const currentQuestionRef = useRef(null);
  const segmentIndexRef = useRef(0);
  const sixSecondShownRef = useRef(false);
  const threeSecondShownRef = useRef(false);
  // Track followup state in refs so event listeners always see current values
  const isFollowupRef = useRef(false);
  const followupTypeRef = useRef(null);

  const { segments, segmentsRef, videoPath } = useSegments(videoId);

  // ── Launch mpv ─────────────────────────────────────────────────────────────
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
        setPiggyMode("watch");
        setPiggyText("Let's watch carefully 👀");
      })
      .catch((e) => {
        console.error("[WatchVideoPanel] launch failed:", e);
        setPiggyText("Failed to launch video");
      });
  }, [videoPath, segments, launched]);

  // ── Gaze ───────────────────────────────────────────────────────────────────
  useEffect(() => {
    invoke("gaze_start").catch(() => {});
    return () => {
      invoke("gaze_stop").catch(() => {});
      invoke("mpv_quit").catch(() => {});
      // Make sure we don't leave the pipeline stuck in AnswerMode on unmount
      invoke("clear_answer_context").catch(() => {});
    };
  }, []);

  useEffect(() => {
    currentQuestionRef.current = currentQuestion;
  }, [currentQuestion]);

  useEffect(() => {
    isFollowupRef.current = isFollowup;
    followupTypeRef.current = followupType;
  }, [isFollowup, followupType]);

  useEffect(() => {
    if (currentQuestion) setLookingAway(false);
  }, [currentQuestion?.question]);

  // ── mpv tick → piggy countdown ─────────────────────────────────────────────
  useEffect(() => {
    let unlisten;
    listen("orb://mpv-tick", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      const pos = data.position;
      const segs = segmentsRef.current;
      const idx = segmentIndexRef.current;
      if (!segs[idx] || currentQuestionRef.current) return;
      const timeLeft = segs[idx].end_seconds - pos;
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

  // ── Segment end → show question ────────────────────────────────────────────
  useEffect(() => {
    let unlisten;
    listen("orb://segment-end", ({ payload }) => {
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
      const question = match || {
        question: segment.best_question || "What did you just see?",
      };
      setCurrentQuestion(question);
      setIsFollowup(false);
      setFollowupType(null);
      setPiggyMode("hidden");
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [segmentsRef]);

  // ── Set answer context whenever question/followup changes ──────────────────
  useEffect(() => {
    if (!currentQuestion) return;
    const segs = segmentsRef.current;
    const segment = segs[segmentIndexRef.current];
    if (!segment) return;

    let expectedAnswer;
    if (isFollowup) {
      expectedAnswer =
        followupType === "correct"
          ? currentQuestion.followup_correct_answer
          : currentQuestion.followup_wrong_answer;
    } else {
      // Find the answer for the best_question
      const match = segment.questions?.find(
        (x) =>
          x.question === segment.best_question ||
          x.question?.includes(segment.best_question),
      );
      expectedAnswer = match?.answer ?? "";
    }

    invoke("set_answer_context", {
      expectedAnswer: expectedAnswer ?? "",
      videoId,
      segmentId: segment.id,
    }).catch(console.error);

    setRecordingState("listening");
    setStatusMessage("Listening for your answer… 🎤");
  }, [currentQuestion, isFollowup, followupType, videoId, segmentsRef]);

  // ── Answer result from Rust ────────────────────────────────────────────────
  useEffect(() => {
    let unlisten;
    listen("orb://answer-result", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      setRecordingState("analyzing");
      setStatusMessage("Checking your answer…");
      // Small delay so "analyzing" state is visible before result renders
      setTimeout(() => handleResult(data), 600);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // ── Navigation helpers ─────────────────────────────────────────────────────
  const advanceAndPlay = useCallback(() => {
    const segs = segmentsRef.current;
    const nextIdx = segmentIndexRef.current + 1;
    segmentIndexRef.current = nextIdx;

    setCurrentQuestion(null);
    setIsFollowup(false);
    setFollowupType(null);
    setRecordingState("idle");
    setStatusMessage("");
    sixSecondShownRef.current = false;
    threeSecondShownRef.current = false;

    invoke("clear_answer_context").catch(() => {});

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

    invoke("mpv_play").catch(console.error);
  }, [segmentsRef, onClose]);

  const replaySegment = useCallback(() => {
    const segs = segmentsRef.current;
    const idx = segmentIndexRef.current;
    if (!segs[idx]) return;

    sixSecondShownRef.current = false;
    threeSecondShownRef.current = false;

    invoke("clear_answer_context").catch(() => {});

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

  // ── Result handler (reads refs so it's safe inside the event listener) ─────
  const handleResult = useCallback(
    (result) => {
      if (!result) {
        advanceAndPlay();
        return;
      }

      const q = currentQuestionRef.current;
      const followup = isFollowupRef.current;
      const fType = followupTypeRef.current;

      if (result.is_correct) {
        setRecordingState("correct");

        if (followup) {
          // Followup correct → always advance
          setStatusMessage("Correct! Well done 🎉");
          setTimeout(() => advanceAndPlay(), 2000);
        } else if (q?.followup_correct_question) {
          setStatusMessage("Correct! Here's a bonus question.");
          setTimeout(() => {
            setRecordingState("idle");
            setStatusMessage("");
            setFollowupType("correct");
            setIsFollowup(true);
            // currentQuestion stays the same — followup answer extracted from it
          }, 2000);
        } else {
          setStatusMessage("Correct! Well done 🎉");
          setTimeout(() => advanceAndPlay(), 2000);
        }
      } else {
        setRecordingState("wrong");

        if (followup) {
          // Followup wrong → replay
          setStatusMessage("Not quite — let's rewatch!");
          setTimeout(() => {
            setCurrentQuestion(null);
            setIsFollowup(false);
            setFollowupType(null);
            setRecordingState("idle");
            setTimeout(() => replaySegment(), 100);
          }, 2000);
        } else if (q?.followup_wrong_question) {
          setStatusMessage("Not quite — try this instead!");
          setTimeout(() => {
            setRecordingState("idle");
            setStatusMessage("");
            setFollowupType("wrong");
            setIsFollowup(true);
          }, 2000);
        } else {
          setStatusMessage("Not quite — let's rewatch!");
          setTimeout(() => {
            setCurrentQuestion(null);
            setRecordingState("idle");
            setTimeout(() => replaySegment(), 100);
          }, 2000);
        }
      }
    },
    [advanceAndPlay, replaySegment],
  );

  const handleCloseQuestion = useCallback(() => {
    invoke("clear_answer_context").catch(() => {});
    setRecordingState("idle");
    setStatusMessage("");
    setCurrentQuestion(null);
    advanceAndPlay();
  }, [advanceAndPlay]);

  // ── Gaze tracker ───────────────────────────────────────────────────────────
  useGazeTracker({
    enabled: true,
    paused: !!currentQuestion,
    onLookAway: () => {
      invoke("mpv_pause").catch(() => {});
      invoke("mpv_minimize").catch(() => {});
      setLookingAway(true);
    },
    onReturn: () => {
      setLookingAway(false);
      if (!currentQuestionRef.current) {
        invoke("mpv_play").catch(() => {});
      }
    },
  });

  const displayQuestion = isFollowup
    ? followupType === "correct"
      ? currentQuestion?.followup_correct_question
      : currentQuestion?.followup_wrong_question
    : currentQuestion?.question;

  return (
    <div className="fixed inset-0 z-50 pointer-events-none">
      {/* Header */}
      <div className="absolute top-0 left-0 right-0 z-10 flex items-center justify-between px-4 py-3 pointer-events-auto">
        <button
          onClick={() => {
            invoke("clear_answer_context").catch(() => {});
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
              recordingState === "listening"
                ? "bg-violet-500/20 text-violet-300 border-violet-500/30"
                : recordingState === "recording"
                  ? "bg-red-500/20 text-red-300 border-red-500/30"
                  : recordingState === "correct"
                    ? "bg-green-500/20 text-green-300 border-green-500/30"
                    : recordingState === "wrong"
                      ? "bg-red-500/20 text-red-300 border-red-500/30"
                      : "bg-black/30 text-white/60 border-white/10"
            }`}
          >
            {recordingState === "listening" && (
              <span className="w-2 h-2 rounded-full bg-violet-400 animate-pulse" />
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

      {/* Look away modal */}
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

          {recordingState === "listening" && (
            <div className="flex items-center gap-2 text-violet-500 text-sm">
              <span className="w-2 h-2 rounded-full bg-violet-400 animate-pulse" />
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

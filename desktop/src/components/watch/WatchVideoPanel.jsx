import { useState, useRef, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useSegments, useGazeTracker, useTauriListener } from "@/hooks";
import PiggyCompanion from "./PiggyCompanion.jsx";
import LookAtScreenModal from "./LookAtScreenModal.jsx";
import QuestionOverlay from "./QuestionOverlay.jsx";

const PAUSE_MESSAGES = [
  "Let's go — what happened? ▶️",
  "Why did we stop? Let's keep watching 👀",
  "Come on, we were doing so good 😄",
];

const WATCH_TEXT = "Let's watch carefully 👀";

export default function WatchVideoPanel({ videoId, onClose }) {
  const [currentQuestion, setCurrentQuestion] = useState(null);
  const [isFollowup, setIsFollowup] = useState(false);
  const [followupType, setFollowupType] = useState(null);
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
  const isFollowupRef = useRef(false);
  const followupTypeRef = useRef(null);
  // Track active timers so we can clear them all on unmount.
  const timersRef = useRef([]);

  const { segments, segmentsRef, videoPath } = useSegments(videoId);

  const addTimer = (fn, ms) => {
    const id = setTimeout(fn, ms);
    timersRef.current.push(id);
    return id;
  };

  // Clear all pending timers on unmount.
  useEffect(() => {
    return () => timersRef.current.forEach(clearTimeout);
  }, []);

  // Keep refs in sync with state.
  useEffect(() => {
    currentQuestionRef.current = currentQuestion;
  }, [currentQuestion]);
  useEffect(() => {
    isFollowupRef.current = isFollowup;
    followupTypeRef.current = followupType;
  }, [isFollowup, followupType]);

  // Clear looking-away when a question appears.
  useEffect(() => {
    if (currentQuestion) setLookingAway(false);
  }, [currentQuestion]);

  // Launch mpv once videoPath and segments are ready.
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
        setPiggyText(WATCH_TEXT);
      })
      .catch((e) => {
        console.error("[WatchVideoPanel] launch failed:", e);
        setPiggyText("Failed to launch video");
      });
  }, [videoPath, segments, launched]);

  // Start/stop gaze detection for the lifetime of this panel.
  useEffect(() => {
    invoke("gaze_start").catch(() => {});
    return () => {
      invoke("gaze_stop").catch(() => {});
      invoke("mpv_quit").catch(() => {});
      invoke("clear_answer_context").catch(() => {});
    };
  }, []);

  // ── Navigation helpers ────────────────────────────────────────────────────

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
    addTimer(() => {
      setPiggyMode("watch");
      setPiggyText(WATCH_TEXT);
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
    addTimer(() => {
      setPiggyMode("watch");
      setPiggyText(WATCH_TEXT);
    }, 2000);
    invoke("mpv_seek", { seconds: segs[idx].start_seconds }).catch(
      console.error,
    );
    invoke("mpv_play").catch(console.error);
  }, [segmentsRef]);

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
          setStatusMessage("Correct! Well done 🎉");
          addTimer(() => advanceAndPlay(), 2000);
        } else if (q?.followup_correct_question) {
          setStatusMessage("Correct! Here's a bonus question.");
          addTimer(() => {
            setRecordingState("idle");
            setStatusMessage("");
            setFollowupType("correct");
            setIsFollowup(true);
          }, 2000);
        } else {
          setStatusMessage("Correct! Well done 🎉");
          addTimer(() => advanceAndPlay(), 2000);
        }
      } else {
        setRecordingState("wrong");
        if (followup) {
          setStatusMessage("Not quite — let's rewatch!");
          addTimer(() => {
            setCurrentQuestion(null);
            setIsFollowup(false);
            setFollowupType(null);
            setRecordingState("idle");
            addTimer(() => replaySegment(), 100);
          }, 2000);
        } else if (q?.followup_wrong_question) {
          setStatusMessage("Not quite — try this instead!");
          addTimer(() => {
            setRecordingState("idle");
            setStatusMessage("");
            setFollowupType("wrong");
            setIsFollowup(true);
          }, 2000);
        } else {
          setStatusMessage("Not quite — let's rewatch!");
          addTimer(() => {
            setCurrentQuestion(null);
            setRecordingState("idle");
            addTimer(() => replaySegment(), 100);
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

  // ── Tauri event listeners ─────────────────────────────────────────────────

  useTauriListener("orb://mpv-tick", (data) => {
    const segs = segmentsRef.current;
    const idx = segmentIndexRef.current;
    if (!segs[idx] || currentQuestionRef.current) return;
    const timeLeft = segs[idx].end_seconds - data.position;
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
  });

  useTauriListener("orb://segment-end", () => {
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
  });

  useTauriListener("orb://answer-result", (data) => {
    setRecordingState("analyzing");
    setStatusMessage("Checking your answer…");
    addTimer(() => handleResult(data), 600);
  });

  // ── Set answer context whenever question/followup changes ─────────────────

  useEffect(() => {
    if (!currentQuestion) return;
    const segment = segmentsRef.current[segmentIndexRef.current];
    if (!segment) return;

    let expectedAnswer;
    if (isFollowup) {
      expectedAnswer =
        followupType === "correct"
          ? currentQuestion.followup_correct_answer
          : currentQuestion.followup_wrong_answer;
    } else {
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
  }, [currentQuestion, isFollowup, followupType, videoId]);

  // ── Gaze tracker ──────────────────────────────────────────────────────────

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
      if (!currentQuestionRef.current) invoke("mpv_play").catch(() => {});
    },
  });

  // ── Derived display values ────────────────────────────────────────────────

  const displayQuestion = isFollowup
    ? followupType === "correct"
      ? currentQuestion?.followup_correct_question
      : currentQuestion?.followup_wrong_question
    : currentQuestion?.question;

  const recordingBadgeClass =
    {
      listening: "bg-violet-500/20 text-violet-300 border-violet-500/30",
      recording: "bg-red-500/20 text-red-300 border-red-500/30",
      correct: "bg-green-500/20 text-green-300 border-green-500/30",
      wrong: "bg-red-500/20 text-red-300 border-red-500/30",
      analyzing: "bg-black/30 text-white/60 border-white/10",
    }[recordingState] ?? "bg-black/30 text-white/60 border-white/10";

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
            className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium border backdrop-blur-sm ${recordingBadgeClass}`}
          >
            {recordingState === "listening" && (
              <span className="w-2 h-2 rounded-full bg-violet-400 animate-pulse" />
            )}
            {statusMessage}
          </div>
        )}

        <div className="w-8" />
      </div>

      {!displayQuestion && !lookingAway && (
        <PiggyCompanion mode={piggyMode} text={piggyText} />
      )}

      {lookingAway && !currentQuestion && (
        <div className="pointer-events-auto">
          <LookAtScreenModal />
        </div>
      )}

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

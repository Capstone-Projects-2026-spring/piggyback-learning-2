"use client";

import { useState, useRef, useCallback, useEffect, useContext } from "react";
import { useParams, useRouter } from "next/navigation";
import YouTube from "react-youtube";

import { useSegments } from "@/hooks/useSegments";
import { useGazeTracker } from "@/hooks/useGazeTracker";
import { useAudioRecorder } from "@/hooks/useAudioRecorder";
import { usePlaybackPoller } from "@/hooks/usePlaybackPoller";
import PiggyCompanion from "@/components/PiggyCompanion";

import QuestionModal from "@/components/QuestionModal";
import LookAtScreenModal from "@/components/LookAtScreenModal";
import RecordingStatusBadge from "@/components/RecordingStatusBadge";
import { useSocket } from "@/context/SocketContext";
import { AuthContext } from "@/context/AuthContext";
import ProtectedRoute from "@/components/ProtectedRoute";

export default function WatchVideoPage() {
  // To prevent the custom hooks from triggering
  return (
    <ProtectedRoute>
      <WatchVideoPageInner />
    </ProtectedRoute>
  );
}

function WatchVideoPageInner() {
  const router = useRouter();

  const { username, send } = useSocket();
  const { parentUsername, role } = useContext(AuthContext);

  const params = useParams();
  const video_id = params.videoId;

  const [currentQuestion, setCurrentQuestion] = useState(null);
  const [segmentIndex, setSegmentIndex] = useState(0);
  const [recordingState, setRecordingState] = useState("idle");
  const [statusMessage, setStatusMessage] = useState("");
  const [analysisResult, setAnalysisResult] = useState(null);
  const [lookingAway, setLookingAway] = useState(false);
  const [piggyMode, setPiggyMode] = useState("watch");
const [piggyText, setPiggyText] = useState("Let’s watch carefully 👀");
const pauseMessages = [
  "Let’s go — what happened? ▶️",
  "Why did we stop? Let’s keep watching 👀",
  "Come on, we were doing so good 😄",
];

  const playerRef = useRef(null);
  const segmentIndexRef = useRef(0);
  const currentQuestionRef = useRef(null);
  const sixSecondShownRef = useRef(false);
const threeSecondShownRef = useRef(false);

  useEffect(() => {
    if (role === "parent") router.push("/");
  }, [role, router]);

  useEffect(() => {
  segmentIndexRef.current = segmentIndex;
  sixSecondShownRef.current = false;
  threeSecondShownRef.current = false;
  setPiggyMode("watch");
  setPiggyText("Let’s watch carefully 👀");
}, [segmentIndex]);

  useEffect(() => {
    currentQuestionRef.current = currentQuestion;
  }, [currentQuestion]);

  // Forcefully dismiss the look away modal if a question pops up
  useEffect(() => {
    if (currentQuestion) {
      setLookingAway(false);
    }
  }, [currentQuestion]);

  const { segmentsRef } = useSegments(video_id);

  const advanceAndPlay = useCallback(() => {
    setCurrentQuestion(null);
    setAnalysisResult(null);
    setRecordingState("idle");
    setStatusMessage("");
    setSegmentIndex((prev) => prev + 1);
    setPiggyMode("talk");
  setPiggyText("Nice! Let’s keep watching 🎬");

  setTimeout(() => {
    setPiggyMode("watch");
    setPiggyText("Let’s watch carefully 👀");
  }, 2000);
    playerRef.current?.playVideo();
  }, []);

  const replaySegment = useCallback(() => {
    const segs = segmentsRef.current;
    const idx = segmentIndexRef.current;
    if (!playerRef.current || idx >= segs.length) return;
    sixSecondShownRef.current = false;
  threeSecondShownRef.current = false;

  setPiggyMode("talk");
  setPiggyText("Let’s try that part again 👀");

  setTimeout(() => {
    setPiggyMode("watch");
    setPiggyText("Let’s watch carefully 👀");
  }, 2000);

    playerRef.current.seekTo(segs[idx].start_seconds ?? 0, true);
    playerRef.current.playVideo();
  }, [segmentsRef]);

  const handleResult = useCallback(
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
          setCurrentQuestion(null);
          setAnalysisResult(null);
          setTimeout(() => replaySegment(), 100);
        }, 2000);
      }
    },
    [advanceAndPlay, replaySegment],
  );

  const recorder = useAudioRecorder({
    onStateChange: setRecordingState,
    onStatusChange: setStatusMessage,
    onResult: handleResult,
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
      // 1. Try to find a fuzzy match just like before
      const match = segment.questions?.find(
        (x) =>
          x.question === segment.best_question ||
          x.question.includes(segment.best_question) ||
          (segment.best_question && segment.best_question.includes(x.question))
      );

      // 2. Fallback safely to best_question (or a default string) so the app NEVER hangs
      setCurrentQuestion(
        match?.question || segment.best_question || "What did you just see?"
      );
    },
  });

  useGazeTracker({
    enabled: true,
    paused: !!currentQuestion, // ONLY pause during question modal, not during lookingAway
    onLookAway: () => {
      send({
        sender: username,
        receiver: parentUsername,
        action: "distracted",
        msg: "Could you help them focus, please?",
      });

      playerRef.current?.pauseVideo();
      setLookingAway(true);
    },
    onReturn: () => {
      send({
        sender: username,
        receiver: parentUsername,
        action: "focused",
        msg: "Hurray!",
      });
      setLookingAway(false);
      playerRef.current?.playVideo();
    },
  });

  // Kick off recording whenever a question appears
  useEffect(() => {
    if (!currentQuestion) return;
    const segment = segmentsRef.current[segmentIndexRef.current];
    if (!segment) return;
    setPiggyMode("hidden");
    recorder.start(segment, {
      kid_id: params.kidId,
      video_id,
      segment_id: segment.id,
    });
    return () => recorder.cancel();
  }, [currentQuestion]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleCloseQuestion = useCallback(() => {
    recorder.cancel();
    setRecordingState("idle");
    setStatusMessage("");
    setAnalysisResult(null);
    advanceAndPlay();
  }, [recorder, advanceAndPlay]);

  if (!video_id) {
    return <p>Loading…</p>;
  }

  return (
    <div className="min-h-screen bg-linear-to-br from-yellow-100 via-pink-100 to-purple-100 flex flex-col items-center p-4">
      <h1 className="text-3xl font-bold text-purple-700 mb-6">
        Watch & Learn!
      </h1>

      <RecordingStatusBadge
        recordingState={recordingState}
        statusMessage={statusMessage}
      />

      <div className="relative w-full max-w-7xl shadow-2xl rounded-xl overflow-hidden">
        <YouTube
          videoId={video_id}
          onReady={(event) => (playerRef.current = event.target)}
          onStateChange={(event) => {
    if (currentQuestion) return;

    if (event.data === 2) {
      const msg =
        pauseMessages[Math.floor(Math.random() * pauseMessages.length)];
      setPiggyMode("talk");
      setPiggyText(msg);
    }

    if (event.data === 1) {
      setPiggyMode("watch");
      setPiggyText("Let’s watch carefully 👀");
    }
  }}
          opts={{
            width: "100%",
            height: "700px",
            playerVars: { autoplay: 1, controls: 1 },
          }}
        />
        <div className="absolute inset-0 flex items-end justify-end pr-24 pb-20 pointer-events-none">
    <PiggyCompanion
      mode={currentQuestion ? "hidden" : piggyMode}
      text={currentQuestion ? "" : piggyText}
    />
  </div>
      </div>

      <QuestionModal
        question={currentQuestion}
        onClose={handleCloseQuestion}
        recordingState={recordingState}
        statusMessage={statusMessage}
        analysisResult={analysisResult}
      />

      <LookAtScreenModal visible={lookingAway} />
    </div>
  );
}

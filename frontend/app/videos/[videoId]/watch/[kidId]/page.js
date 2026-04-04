"use client";

import { useState, useRef, useCallback, useEffect, useContext } from "react";
import { useParams, useRouter } from "next/navigation";
import YouTube from "react-youtube";

import { useSegments } from "@/hooks/useSegments";
import { useGazeTracker } from "@/hooks/useGazeTracker";
import { useAudioRecorder } from "@/hooks/useAudioRecorder";
import { usePlaybackPoller } from "@/hooks/usePlaybackPoller";

import QuestionModal from "@/components/QuestionModal";
import LookAtScreenModal from "@/components/LookAtScreenModal";
import RecordingStatusBadge from "@/components/RecordingStatusBadge";
import { useSocket } from "@/context/SocketContext";
import { AuthContext } from "@/context/AuthContext";

export default function WatchVideoPage() {
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

  const playerRef = useRef(null);
  const segmentIndexRef = useRef(0);
  const currentQuestionRef = useRef(null);

  useEffect(() => {
    if (role === "parent") router.push("/");
  }, [role, router]);

  useEffect(() => {
    segmentIndexRef.current = segmentIndex;
  }, [segmentIndex]);
  useEffect(() => {
    currentQuestionRef.current = currentQuestion;
  }, [currentQuestion]);

  const { segmentsRef } = useSegments(video_id);

  const advanceAndPlay = useCallback(() => {
    setCurrentQuestion(null);
    setAnalysisResult(null);
    setRecordingState("idle");
    setStatusMessage("");
    setSegmentIndex((prev) => prev + 1);
    playerRef.current?.playVideo();
  }, []);

  const replaySegment = useCallback(() => {
    const segs = segmentsRef.current;
    const idx = segmentIndexRef.current;
    if (!playerRef.current || idx >= segs.length) return;
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
    onSegmentEnd: (segment) =>
      setCurrentQuestion(
        segment.questions.filter(
          (x) =>
            x.question === segment.best_question ||
            x.question.includes(segment.best_question),
        )[0]?.question,
      ),
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

  if (!video_id) return <p>Loading…</p>;

  return (
    <div className="min-h-screen bg-linear-to-br from-yellow-100 via-pink-100 to-purple-100 flex flex-col items-center p-4">
      <h1 className="text-3xl font-bold text-purple-700 mb-6">
        Watch & Learn!
      </h1>

      <RecordingStatusBadge
        recordingState={recordingState}
        statusMessage={statusMessage}
      />

      <div className="w-full max-w-7xl shadow-2xl rounded-xl overflow-hidden">
        <YouTube
          videoId={video_id}
          onReady={(event) => (playerRef.current = event.target)}
          opts={{
            width: "100%",
            height: "700px",
            playerVars: { autoplay: 1, controls: 1 },
          }}
        />
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

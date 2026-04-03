"use client";

import { useState, useRef, useCallback, useEffect } from "react";
import { useParams } from "next/navigation";
import YouTube from "react-youtube";
import QuestionModal from "@/components/QuestionModal";
import { usePlaybackPoller } from "./_hooks/usePlaybackPoller";
import { useAudioRecorder } from "./_hooks/useAudioRecorder";
import RecordingStatusBadge from "@/components/RecordingStatusBadge";
import { useSegments } from "./_hooks/useSegment";

export default function WatchVideoPage() {
  const params = useParams();
  const video_id = params.videoId;

  const [currentQuestion, setCurrentQuestion] = useState(null);
  const [segmentIndex, setSegmentIndex] = useState(0);
  const [recordingState, setRecordingState] = useState("idle");
  const [statusMessage, setStatusMessage] = useState("");
  const [analysisResult, setAnalysisResult] = useState(null);

  const playerRef = useRef(null);
  const segmentIndexRef = useRef(0);
  const currentQuestionRef = useRef(null);

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
    onSegmentEnd: (segment) => setCurrentQuestion(segment.best_question),
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
    </div>
  );
}

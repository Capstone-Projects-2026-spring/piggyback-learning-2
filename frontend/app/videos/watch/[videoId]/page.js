"use client";

import { useEffect, useState, useRef } from "react";
import { useParams } from "next/navigation";
import YouTube from "react-youtube";
import QuestionModal from "@/components/QuestionModal";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function WatchVideoPage() {
  const params = useParams();
  const video_id = params.videoId;

  const [segments, setSegments] = useState([]);
  const [currentQuestion, setCurrentQuestion] = useState(null);
  const [segmentIndex, setSegmentIndex] = useState(0);

  const playerRef = useRef(null);
  const intervalRef = useRef(null);

  // Keep refs in sync with state so the interval always sees fresh values
  const segmentIndexRef = useRef(0);
  const currentQuestionRef = useRef(null);
  const segmentsRef = useRef([]);

  useEffect(() => {
    segmentsRef.current = segments;
  }, [segments]);
  useEffect(() => {
    segmentIndexRef.current = segmentIndex;
  }, [segmentIndex]);
  useEffect(() => {
    currentQuestionRef.current = currentQuestion;
  }, [currentQuestion]);

  useEffect(() => {
    if (!video_id) return;
    fetch(`${BASE_URL}/api/questions/${video_id}`)
      .then((res) => res.json())
      .then((data) => setSegments(data.segments))
      .catch((err) => console.error(err));
  }, [video_id]);

  // Register the interval exactly once — refs keep it from going stale
  useEffect(() => {
    intervalRef.current = setInterval(() => {
      const player = playerRef.current;
      const segs = segmentsRef.current;
      const idx = segmentIndexRef.current;

      if (!player || segs.length === 0) return;
      if (idx >= segs.length) {
        clearInterval(intervalRef.current);
        return;
      }

      const segment = segs[idx];
      const currentTime = player.getCurrentTime();

      if (currentTime >= segment.end_seconds && !currentQuestionRef.current) {
        player.pauseVideo();
        setCurrentQuestion(segment.best_question);
      }
    }, 250);

    return () => clearInterval(intervalRef.current);
  }, []);

  const handleCloseQuestion = (answer) => {
    console.log("User answer:", answer);
    setCurrentQuestion(null);
    setSegmentIndex((prev) => prev + 1);
    playerRef.current.playVideo();
  };

  if (!video_id) return <p>Loading...</p>;

  return (
    <div className="min-h-screen bg-linear-to-br from-yellow-100 via-pink-100 to-purple-100 flex flex-col items-center p-4">
      <h1 className="text-3xl font-bold text-purple-700 mb-6">
        Watch & Learn!
      </h1>
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
      <QuestionModal question={currentQuestion} onClose={handleCloseQuestion} />
    </div>
  );
}

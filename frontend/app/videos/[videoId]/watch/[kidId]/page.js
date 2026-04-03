"use client";
import { useEffect, useState, useRef, useCallback } from "react";
import { useParams } from "next/navigation";
import YouTube from "react-youtube";
import QuestionModal from "@/components/QuestionModal";

// --- WAV encoding helpers ---
function floatTo16BitPCM(output, offset, input) {
  for (let i = 0; i < input.length; i++, offset += 2) {
    const s = Math.max(-1, Math.min(1, input[i]));
    output.setInt16(offset, s < 0 ? s * 0x8000 : s * 0x7fff, true);
  }
}

function encodeWAV(samples, sampleRate) {
  const buffer = new ArrayBuffer(44 + samples.length * 2);
  const view = new DataView(buffer);
  const writeString = (offset, str) => {
    for (let i = 0; i < str.length; i++)
      view.setUint8(offset + i, str.charCodeAt(i));
  };
  writeString(0, "RIFF");
  view.setUint32(4, 36 + samples.length * 2, true);
  writeString(8, "WAVE");
  writeString(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true); // PCM format
  view.setUint16(22, 1, true); // mono
  view.setUint32(24, sampleRate, true); // 16000 Hz
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  writeString(36, "data");
  view.setUint32(40, samples.length * 2, true);
  floatTo16BitPCM(view, 44, samples);
  return new Blob([buffer], { type: "audio/wav" });
}

function downsampleBuffer(buffer, inputRate, outputRate) {
  if (inputRate === outputRate) return buffer;
  const ratio = inputRate / outputRate;
  const newLength = Math.round(buffer.length / ratio);
  const result = new Float32Array(newLength);
  for (let i = 0; i < newLength; i++) {
    result[i] = buffer[Math.round(i * ratio)];
  }
  return result;
}

const TARGET_SAMPLE_RATE = 16000;
const RECORD_DELAY_MS = 3000;
const RECORD_DURATION_MS = 5000;

export default function WatchVideoPage() {
  const params = useParams();
  const video_id = params.videoId;

  const [segments, setSegments] = useState([]);
  const [currentQuestion, setCurrentQuestion] = useState(null);
  const [segmentIndex, setSegmentIndex] = useState(0);
  const [recordingState, setRecordingState] = useState("idle"); // idle | waiting | recording | analyzing
  const [statusMessage, setStatusMessage] = useState("");

  const playerRef = useRef(null);
  const intervalRef = useRef(null);
  const segmentsRef = useRef([]);
  const segmentIndexRef = useRef(0);
  const currentQuestionRef = useRef(null);
  const recordingTimeoutsRef = useRef([]);

  useEffect(() => {
    segmentsRef.current = segments;
  }, [segments]);
  useEffect(() => {
    segmentIndexRef.current = segmentIndex;
  }, [segmentIndex]);
  useEffect(() => {
    currentQuestionRef.current = currentQuestion;
  }, [currentQuestion]);

  // Fetch segments
  useEffect(() => {
    if (!video_id) return;
    fetch(`${process.env.NEXT_PUBLIC_API_BASE_URL}/api/questions/${video_id}`)
      .then((res) => res.json())
      .then((data) => setSegments(data.segments))
      .catch((err) => console.error(err));
  }, [video_id]);

  // Playback polling — register once, refs keep values fresh
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

  const clearRecordingTimeouts = useCallback(() => {
    recordingTimeoutsRef.current.forEach(clearTimeout);
    recordingTimeoutsRef.current = [];
  }, []);

  const advanceAndPlay = useCallback(() => {
    setCurrentQuestion(null);
    setSegmentIndex((prev) => prev + 1);
    playerRef.current?.playVideo();
  }, []);

  const replaySegment = useCallback(() => {
    const segs = segmentsRef.current;
    const idx = segmentIndexRef.current;
    if (!playerRef.current || idx >= segs.length) return;
    const startTime = segs[idx].start_seconds ?? 0;
    playerRef.current.seekTo(startTime, true);
    playerRef.current.playVideo();
  }, []);

  const handleAnalysisResult = useCallback(
    (correct) => {
      if (correct) {
        setRecordingState("correct");
        setStatusMessage("Correct! Well done 🎉");
        setTimeout(() => advanceAndPlay(), 1500);
      } else {
        setRecordingState("wrong");
        setStatusMessage("Not quite — let's rewatch!");
        setTimeout(() => {
          setCurrentQuestion(null);
          setTimeout(() => replaySegment(), 100);
        }, 1500);
      }
    },
    [advanceAndPlay, replaySegment],
  );

  // Trigger recording when question appears — all setState deferred via setTimeout(0)
  useEffect(() => {
    if (!currentQuestion) return;
    const segment = segmentsRef.current[segmentIndexRef.current];
    if (!segment) return;
    console.log(segment);

    clearRecordingTimeouts();

    const outerTimeout = setTimeout(async () => {
      setRecordingState("waiting");
      setStatusMessage("Get ready to answer in 3 seconds…");

      await new Promise((resolve) => {
        const t = setTimeout(resolve, RECORD_DELAY_MS);
        recordingTimeoutsRef.current.push(t);
      });

      let stream;
      try {
        stream = await navigator.mediaDevices.getUserMedia({
          audio: { channelCount: 1, sampleRate: TARGET_SAMPLE_RATE },
        });
      } catch (err) {
        console.error("Microphone access denied:", err);
        setStatusMessage("Microphone access denied.");
        setRecordingState("idle");
        return;
      }

      const audioCtx = new AudioContext({ sampleRate: TARGET_SAMPLE_RATE });
      const source = audioCtx.createMediaStreamSource(stream);
      const processor = audioCtx.createScriptProcessor(4096, 1, 1);
      const chunks = [];

      source.connect(processor);
      processor.connect(audioCtx.destination);
      processor.onaudioprocess = (e) => {
        chunks.push(new Float32Array(e.inputBuffer.getChannelData(0)));
      };

      setRecordingState("recording");
      setStatusMessage("🎙️ Recording your answer…");

      await new Promise((resolve) => {
        const t = setTimeout(resolve, RECORD_DURATION_MS);
        recordingTimeoutsRef.current.push(t);
      });

      processor.disconnect();
      source.disconnect();
      audioCtx.close();
      stream.getTracks().forEach((t) => t.stop());

      const totalLength = chunks.reduce((sum, c) => sum + c.length, 0);
      const merged = new Float32Array(totalLength);
      let offset = 0;
      for (const chunk of chunks) {
        merged.set(chunk, offset);
        offset += chunk.length;
      }

      const downsampled = downsampleBuffer(
        merged,
        audioCtx.sampleRate,
        TARGET_SAMPLE_RATE,
      );
      const wavBlob = encodeWAV(downsampled, TARGET_SAMPLE_RATE);

      setRecordingState("analyzing");
      setStatusMessage("Analyzing your answer…");

      try {
        const formData = new FormData();
        formData.append("audio", wavBlob, "audio_file.wav");
        formData.append(
          "expected_answer",
          segment.questions.filter(
            (x) =>
              x.question == segment.best_question ||
              x.question.includes(segment.best_question),
          )[0]?.answer,
        );
        formData.append("kid_id", params.kidId);
        formData.append("video_id", video_id);
        formData.append("segment_id", segment.id);

        const res = await fetch(
          `${process.env.NEXT_PUBLIC_API_BASE_URL}/api/answers/analyze`,
          { method: "POST", body: formData },
        );
        const result = await res.json();
        handleAnalysisResult(result.correct);
      } catch (err) {
        console.error("Analysis failed:", err);
        setStatusMessage("Analysis failed. Continuing…");
        setRecordingState("idle");
        advanceAndPlay();
      }
    }, 0);

    recordingTimeoutsRef.current.push(outerTimeout);

    return () => clearRecordingTimeouts();
  }, [currentQuestion]); // eslint-disable-line react-hooks/exhaustive-deps

  // Manual dismiss fallback
  const handleCloseQuestion = useCallback(() => {
    clearRecordingTimeouts();
    setRecordingState("idle");
    setStatusMessage("");
    advanceAndPlay();
  }, [clearRecordingTimeouts, advanceAndPlay]);

  if (!video_id) return <p>Loading…</p>;

  return (
    <div className="min-h-screen bg-linear-to-br from-yellow-100 via-pink-100 to-purple-100 flex flex-col items-center p-4">
      <h1 className="text-3xl font-bold text-purple-700 mb-6">
        Watch & Learn!
      </h1>

      {recordingState !== "idle" && (
        <div
          className={`mb-4 px-4 py-2 rounded-full text-sm font-semibold shadow-md transition-all ${
            recordingState === "waiting"
              ? "bg-yellow-300 text-yellow-900"
              : recordingState === "recording"
                ? "bg-red-500 text-white animate-pulse"
                : "bg-blue-400 text-white"
          }`}
        >
          {statusMessage}
        </div>
      )}

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
      />
    </div>
  );
}

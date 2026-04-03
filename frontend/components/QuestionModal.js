import { useEffect, useRef } from "react";

const BAR_COUNT = 20;

export default function QuestionModal({
  question,
  onClose,
  recordingState,
  statusMessage,
  analysisResult,
}) {
  const canvasRef = useRef(null);
  const animFrameRef = useRef(null);

  useEffect(() => {
    if (recordingState !== "recording") {
      cancelAnimationFrame(animFrameRef.current);
      const canvas = canvasRef.current;
      if (canvas) {
        const ctx = canvas.getContext("2d");
        ctx.clearRect(0, 0, canvas.width, canvas.height);
      }
      return;
    }

    let audioCtx;
    let streamRef;

    (async () => {
      try {
        streamRef = await navigator.mediaDevices.getUserMedia({ audio: true });
        audioCtx = new AudioContext();
        const source = audioCtx.createMediaStreamSource(streamRef);
        const analyser = audioCtx.createAnalyser();
        analyser.fftSize = 64;
        source.connect(analyser);

        const canvas = canvasRef.current;
        const ctx = canvas.getContext("2d");
        const dataArray = new Uint8Array(analyser.frequencyBinCount);

        const draw = () => {
          animFrameRef.current = requestAnimationFrame(draw);
          analyser.getByteFrequencyData(dataArray);
          ctx.clearRect(0, 0, canvas.width, canvas.height);
          const barWidth = canvas.width / BAR_COUNT - 2;
          for (let i = 0; i < BAR_COUNT; i++) {
            const value = dataArray[i] / 255;
            const barHeight = value * canvas.height;
            const hue = 260 + value * 60;
            ctx.fillStyle = `hsl(${hue}, 80%, 60%)`;
            ctx.beginPath();
            ctx.roundRect(
              i * (barWidth + 2),
              canvas.height - barHeight,
              barWidth,
              barHeight,
              4,
            );
            ctx.fill();
          }
        };
        draw();
      } catch (e) {
        console.error("Visualizer mic error:", e);
      }
    })();

    return () => {
      cancelAnimationFrame(animFrameRef.current);
      streamRef?.getTracks().forEach((t) => t.stop());
      audioCtx?.close();
    };
  }, [recordingState]);

  if (!question) return null;

  const isCorrect = recordingState === "correct";
  const isWrong = recordingState === "wrong";
  const isDone = isCorrect || isWrong;

  const similarityPct =
    analysisResult?.similarity_score != null
      ? Math.round(analysisResult.similarity_score * 100)
      : null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white p-6 rounded-2xl shadow-lg max-w-md w-full text-center">
        <h2 className="text-2xl font-bold mb-4 text-purple-700">
          Question Time!
        </h2>
        <p className="text-lg mb-6 text-gray-800">{question}</p>

        {/* Waiting */}
        {recordingState === "waiting" && (
          <div className="flex flex-col items-center gap-3 mb-4">
            <div className="text-5xl animate-bounce">🎙️</div>
            <p className="text-gray-500 text-sm">{statusMessage}</p>
          </div>
        )}

        {/* Recording — live waveform */}
        {recordingState === "recording" && (
          <div className="flex flex-col items-center gap-3 mb-4">
            <canvas
              ref={canvasRef}
              width={340}
              height={80}
              className="rounded-xl bg-purple-50 w-full"
            />
            <p className="text-red-500 font-semibold text-sm animate-pulse">
              {statusMessage}
            </p>
          </div>
        )}

        {/* Analyzing */}
        {recordingState === "analyzing" && (
          <div className="flex flex-col items-center gap-3 mb-4">
            <div className="w-10 h-10 border-4 border-purple-400 border-t-transparent rounded-full animate-spin" />
            <p className="text-gray-500 text-sm">{statusMessage}</p>
          </div>
        )}

        {/* Result */}
        {isDone && (
          <div
            className={`flex flex-col items-center gap-3 mb-4 p-4 rounded-xl ${isCorrect ? "bg-green-50" : "bg-red-50"}`}
          >
            <span className="text-4xl">{isCorrect ? "✅" : "❌"}</span>
            <p
              className={`font-bold text-lg ${isCorrect ? "text-green-600" : "text-red-600"}`}
            >
              {statusMessage}
            </p>

            {/* Transcript */}
            {analysisResult?.transcript && (
              <div className="w-full bg-white border border-gray-200 rounded-lg px-3 py-2 text-sm text-gray-700">
                <span className="font-semibold text-gray-400 uppercase text-xs tracking-wide">
                  You said
                </span>
                <p className="mt-1 italic">
                  &quot;{analysisResult.transcript}&quot;
                </p>
              </div>
            )}

            {/* Similarity bar */}
            {similarityPct != null && (
              <div className="w-full">
                <div className="flex justify-between text-xs text-gray-400 mb-1">
                  <span>Match</span>
                  <span>{similarityPct}%</span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div
                    className={`h-2 rounded-full transition-all duration-700 ${isCorrect ? "bg-green-400" : "bg-red-400"}`}
                    style={{ width: `${similarityPct}%` }}
                  />
                </div>
              </div>
            )}

            {isWrong && (
              <p className="text-xs text-gray-400 mt-1">
                Replaying the segment…
              </p>
            )}
          </div>
        )}

        <button
          onClick={() => onClose()}
          className="mt-2 text-xs text-gray-400 hover:text-gray-600 underline transition"
        >
          Skip
        </button>
      </div>
    </div>
  );
}

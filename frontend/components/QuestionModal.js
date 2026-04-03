import { useEffect, useRef } from "react";

const BAR_COUNT = 20;

export default function QuestionModal({
  question,
  onClose,
  recordingState,
  statusMessage,
}) {
  const canvasRef = useRef(null);
  const animFrameRef = useRef(null);
  const analyserRef = useRef(null);
  const streamRef = useRef(null);

  // Visualizer: spin up when recording, tear down otherwise
  useEffect(() => {
    if (recordingState !== "recording") {
      cancelAnimationFrame(animFrameRef.current);
      // Clear canvas
      const canvas = canvasRef.current;
      if (canvas) {
        const ctx = canvas.getContext("2d");
        ctx.clearRect(0, 0, canvas.width, canvas.height);
      }
      return;
    }

    let audioCtx;
    (async () => {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          audio: true,
        });
        streamRef.current = stream;
        audioCtx = new AudioContext();
        const source = audioCtx.createMediaStreamSource(stream);
        const analyser = audioCtx.createAnalyser();
        analyser.fftSize = 64;
        source.connect(analyser);
        analyserRef.current = analyser;

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
            const hue = 260 + value * 60; // purple → pink
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
      streamRef.current?.getTracks().forEach((t) => t.stop());
      audioCtx?.close();
    };
  }, [recordingState]);

  if (!question) return null;

  const isCorrect = recordingState === "correct";
  const isWrong = recordingState === "wrong";
  const isDone = isCorrect || isWrong;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white p-6 rounded-2xl shadow-lg max-w-md w-full text-center">
        <h2 className="text-2xl font-bold mb-4 text-purple-700">
          Question Time!
        </h2>
        <p className="text-lg mb-6 text-gray-800">{question}</p>

        {/* Phase: waiting */}
        {recordingState === "waiting" && (
          <div className="flex flex-col items-center gap-3 mb-4">
            <div className="text-5xl animate-bounce">🎙️</div>
            <p className="text-gray-500 text-sm">{statusMessage}</p>
          </div>
        )}

        {/* Phase: recording — live waveform */}
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

        {/* Phase: analyzing */}
        {recordingState === "analyzing" && (
          <div className="flex flex-col items-center gap-3 mb-4">
            <div className="w-10 h-10 border-4 border-purple-400 border-t-transparent rounded-full animate-spin" />
            <p className="text-gray-500 text-sm">{statusMessage}</p>
          </div>
        )}

        {/* Phase: result */}
        {isDone && (
          <div
            className={`flex flex-col items-center gap-2 mb-4 p-4 rounded-xl ${
              isCorrect ? "bg-green-50" : "bg-red-50"
            }`}
          >
            <span className="text-4xl">{isCorrect ? "✅" : "❌"}</span>
            <p
              className={`font-bold text-lg ${isCorrect ? "text-green-600" : "text-red-600"}`}
            >
              {statusMessage}
            </p>
            {isWrong && (
              <p className="text-sm text-gray-500">Replaying the segment…</p>
            )}
          </div>
        )}

        {/* Manual skip — always available */}
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

import Spinner from "../ui/Spinner.jsx";

export default function QuestionOverlay({
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
              <Spinner className="w-4 h-4 border-violet-300 border-t-violet-500" />
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

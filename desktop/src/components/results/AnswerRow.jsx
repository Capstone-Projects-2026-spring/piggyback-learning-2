import Checkmark from "../ui/Checkmark.jsx";

export const MOOD_EMOJI = {
  happy: "😊",
  sad: "😢",
  angry: "😠",
  surprised: "😲",
  neutral: "😐",
  fearful: "😨",
  disgusted: "😒",
  contempt: "😏",
};

export default function AnswerRow({ answer }) {
  return (
    <div
      className={`flex items-start gap-3 p-3 rounded-2xl ${
        answer.is_correct
          ? "bg-green-50 border border-green-100"
          : "bg-red-50 border border-red-100"
      }`}
    >
      <div
        className={`mt-0.5 w-6 h-6 rounded-full flex items-center justify-center shrink-0 ${
          answer.is_correct ? "bg-green-400" : "bg-red-400"
        }`}
      >
        {answer.is_correct ? (
          <Checkmark className="w-3 h-3 text-white" strokeWidth={3} />
        ) : (
          <span className="text-white text-xs font-bold">✗</span>
        )}
      </div>
      <div className="flex-1 min-w-0">
        <p
          className={`text-sm font-medium leading-snug ${
            answer.is_correct ? "text-green-800" : "text-red-800"
          }`}
        >
          "{answer.transcript || "No answer"}"
        </p>
        <div className="flex items-center gap-2 mt-1">
          <span className="text-xs text-gray-400">
            {(answer.similarity_score * 100).toFixed(0)}% match
          </span>
          {answer.mood && (
            <span className="text-xs text-gray-400">
              · {MOOD_EMOJI[answer.mood] ?? "😐"} {answer.mood}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

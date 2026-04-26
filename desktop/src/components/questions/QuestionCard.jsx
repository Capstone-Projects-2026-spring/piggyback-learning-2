import EditableField from "../ui/EditableField.jsx";
import QuestionFollowUps from "./QuestionFollowUps.jsx";
import { QTYPE_LABELS, QTYPE_COLORS } from "@/utils";

export default function QuestionCard({
  q,
  isBest,
  isExpanded,
  onSetBest,
  onToggleExpand,
  onUpdate,
}) {
  return (
    <div
      className={`rounded-2xl border transition-all duration-200 ${isBest ? "border-pink-300 shadow-sm" : "border-gray-100"}`}
    >
      <div className="flex items-start gap-3 p-4">
        <span
          className={`shrink-0 text-xs px-2 py-0.5 rounded-full border font-medium ${QTYPE_COLORS[q.qtype] ?? "bg-gray-50 border-gray-200 text-gray-500"}`}
        >
          {QTYPE_LABELS[q.qtype] ?? q.qtype}
        </span>

        <div className="flex-1 min-w-0 flex flex-col gap-1">
          <EditableField
            value={q.question}
            multiline
            onChange={(v) => onUpdate("question", v)}
            className="text-gray-800 font-medium leading-snug"
          />
          <div className="flex items-center gap-1 text-xs text-gray-400">
            <span>Answer:</span>
            <EditableField
              value={q.answer}
              onChange={(v) => onUpdate("answer", v)}
              className="font-medium text-gray-600"
            />
          </div>
        </div>

        <div className="flex flex-col items-end gap-2 shrink-0">
          <button
            onClick={onSetBest}
            title={isBest ? "Current best" : "Set as best"}
            className={`text-base transition-opacity ${isBest ? "opacity-100" : "opacity-30 hover:opacity-70"}`}
          >
            ⭐
          </button>
          <button
            onClick={onToggleExpand}
            className="text-gray-300 hover:text-gray-400 transition-colors"
          >
            <svg
              className={`w-4 h-4 transition-transform duration-200 ${isExpanded ? "rotate-180" : ""}`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 9l-7 7-7-7"
              />
            </svg>
          </button>
        </div>
      </div>
      {isExpanded && (
        <QuestionFollowUps q={q} onUpdate={(field, v) => onUpdate(field, v)} />
      )}
    </div>
  );
}

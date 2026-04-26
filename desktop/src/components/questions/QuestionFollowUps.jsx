import EditableField from "../ui/EditableField.jsx";

function FollowUpBlock({
  label,
  colorClasses,
  question,
  answer,
  onChangeQuestion,
  onChangeAnswer,
}) {
  return (
    <div
      className={`rounded-xl border px-3 py-2.5 flex flex-col gap-1.5 ${colorClasses.bg}`}
    >
      <p className={`text-xs font-medium ${colorClasses.label}`}>{label}</p>
      <EditableField
        value={question ?? ""}
        multiline
        onChange={onChangeQuestion}
        className={`text-sm leading-snug ${colorClasses.text}`}
      />
      <div className={`flex items-center gap-1 text-xs ${colorClasses.label}`}>
        <span>Answer:</span>
        <EditableField
          value={answer ?? ""}
          onChange={onChangeAnswer}
          className={`font-medium ${colorClasses.answer}`}
        />
      </div>
    </div>
  );
}

export default function QuestionFollowUps({ q, onUpdate }) {
  return (
    <div className="px-4 pb-4 flex flex-col gap-3 border-t border-gray-50">
      <p className="text-xs text-gray-400 font-medium uppercase tracking-wide mt-3">
        Follow-up Questions
      </p>
      <FollowUpBlock
        label="✓ If correct"
        colorClasses={{
          bg: "bg-green-50 border-green-100",
          label: "text-green-500",
          text: "text-green-800",
          answer: "text-green-700",
        }}
        question={q.followup_correct_question}
        answer={q.followup_correct_answer}
        onChangeQuestion={(v) => onUpdate("followup_correct_question", v)}
        onChangeAnswer={(v) => onUpdate("followup_correct_answer", v)}
      />
      <FollowUpBlock
        label="✗ If wrong"
        colorClasses={{
          bg: "bg-red-50 border-red-100",
          label: "text-red-400",
          text: "text-red-800",
          answer: "text-red-700",
        }}
        question={q.followup_wrong_question}
        answer={q.followup_wrong_answer}
        onChangeQuestion={(v) => onUpdate("followup_wrong_question", v)}
        onChangeAnswer={(v) => onUpdate("followup_wrong_answer", v)}
      />
    </div>
  );
}

import AnswerRow from "./AnswerRow.jsx";

export default function SegmentSection({ segmentId, answers }) {
  const correct = answers.filter((a) => a.is_correct).length;
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between px-1">
        <p className="text-xs font-medium text-gray-500 uppercase tracking-wide">
          Segment {segmentId}
        </p>
        <span
          className={`text-xs font-medium px-2 py-0.5 rounded-full ${
            correct === answers.length
              ? "bg-green-100 text-green-600"
              : correct > 0
                ? "bg-amber-100 text-amber-600"
                : "bg-red-100 text-red-500"
          }`}
        >
          {correct}/{answers.length}
        </span>
      </div>
      {answers.map((ans, i) => (
        <AnswerRow key={i} answer={ans} />
      ))}
    </div>
  );
}

import SegmentSection from "./SegmentSection.jsx";

export default function VideoSection({ videoId, answers }) {
  const total = answers.length;
  const correct = answers.filter((a) => a.is_correct).length;
  const accuracy = total ? Math.round((correct / total) * 100) : 0;

  const bySegment = answers.reduce((acc, ans) => {
    if (!acc[ans.segment_id]) acc[ans.segment_id] = [];
    acc[ans.segment_id].push(ans);
    return acc;
  }, {});

  return (
    <div className="bg-white rounded-2xl border border-gray-100 shadow-sm overflow-hidden">
      <div className="px-4 py-3 border-b border-gray-50 flex items-center justify-between">
        <p className="text-sm font-semibold text-gray-700 truncate max-w-[60%]">
          🎬 {videoId}
        </p>
        <span
          className={`text-xs font-medium px-2 py-0.5 rounded-full ${
            accuracy >= 80
              ? "bg-green-100 text-green-600"
              : accuracy >= 50
                ? "bg-amber-100 text-amber-600"
                : "bg-red-100 text-red-500"
          }`}
        >
          {accuracy}% · {correct}/{total}
        </span>
      </div>
      <div className="p-4 flex flex-col gap-4">
        {Object.entries(bySegment).map(([segId, segAnswers]) => (
          <SegmentSection key={segId} segmentId={segId} answers={segAnswers} />
        ))}
      </div>
    </div>
  );
}

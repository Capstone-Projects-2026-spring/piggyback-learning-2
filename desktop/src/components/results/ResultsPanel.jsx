import CloseButton from "../ui/CloseButton.jsx";
import ScoreBadge from "./ScoreBadge.jsx";
import ProgressBar from "./ProgressBar.jsx";
import VideoSection from "./VideoSection.jsx";

export default function ResultsPanel({ answers, onClose }) {
  const total = answers.length;
  const correct = answers.filter((a) => a.is_correct).length;
  const accuracy = total ? Math.round((correct / total) * 100) : 0;

  const byVideo = answers.reduce((acc, ans) => {
    if (!acc[ans.video_id]) acc[ans.video_id] = [];
    acc[ans.video_id].push(ans);
    return acc;
  }, {});

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto bg-linear-to-b from-violet-50 to-white">
      <div className="px-5 pt-8 pb-16 flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-bold text-gray-800">Results</h2>
          <CloseButton
            onClick={onClose}
            className="w-9 h-9 bg-white border border-gray-100 shadow-sm text-gray-400 hover:text-gray-600"
          />
        </div>

        <div className="px-5 py-4 bg-white rounded-3xl border border-gray-100 shadow-sm">
          <div className="flex items-center justify-between">
            <div className="flex flex-col gap-1">
              <p className="text-xs text-gray-400 uppercase tracking-wide font-medium">
                Overall accuracy
              </p>
              <ScoreBadge accuracy={accuracy} />
            </div>
            <div className="flex flex-col gap-1 text-right">
              <p className="text-sm text-gray-500">
                <span className="text-green-600 font-semibold">{correct}</span>{" "}
                correct
              </p>
              <p className="text-sm text-gray-500">
                <span className="text-red-500 font-semibold">
                  {total - correct}
                </span>{" "}
                wrong
              </p>
              <p className="text-xs text-gray-400">
                {Object.keys(byVideo).length} video
                {Object.keys(byVideo).length !== 1 ? "s" : ""}
              </p>
            </div>
          </div>
          <ProgressBar accuracy={accuracy} />
        </div>

        {total === 0 ? (
          <div className="flex flex-col items-center gap-3 pt-12">
            <p className="text-2xl">📋</p>
            <p className="text-sm text-gray-400 text-center">
              No answers recorded yet.
            </p>
          </div>
        ) : (
          Object.entries(byVideo).map(([videoId, videoAnswers]) => (
            <VideoSection
              key={videoId}
              videoId={videoId}
              answers={videoAnswers}
            />
          ))
        )}
      </div>
    </div>
  );
}

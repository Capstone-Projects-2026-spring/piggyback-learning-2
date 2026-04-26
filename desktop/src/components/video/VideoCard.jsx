import { STAGE_CONFIG, formatDuration } from "@/utils";
import VideoStatusBadge from "./VideoStatusBadge.jsx";
import PipelineSteps from "./PipelineSteps.jsx";
import Checkmark from "../ui/Checkmark.jsx";

export default function VideoCard({
  video,
  status,
  processingInfo,
  hasQuestions,
  isRecommendation,
  isAssigned,
  onViewQuestions,
  onWatch,
}) {
  const duration = video.duration ? formatDuration(video.duration) : null;
  const stage = processingInfo?.stage;
  const stageConfig = stage ? STAGE_CONFIG[stage] : null;
  const progress = processingInfo?.progress;
  const isDownloaded = status === "done" || status === "already_exists";

  return (
    <div className="min-w-full px-5">
      <div className="rounded-2xl overflow-hidden bg-white border border-gray-100 shadow-sm">
        <div className="relative">
          <img
            src={video.thumbnail}
            alt={video.title}
            className="w-full h-48 object-cover"
          />

          {duration && (
            <span className="absolute bottom-2 right-2 bg-black/70 text-white text-xs px-1.5 py-0.5 rounded">
              {duration}
            </span>
          )}
          {video.isYoutube && (
            <span className="absolute top-2 left-2 bg-red-600/90 text-white text-xs px-2 py-0.5 rounded-full font-medium">
              YouTube
            </span>
          )}
          {status === "downloading" && (
            <div className="absolute inset-0 bg-black/40 flex items-center justify-center">
              <div className="w-8 h-8 rounded-full border-2 border-white/40 border-t-white animate-spin" />
            </div>
          )}
          {/* Brief green flash after download, before processing pipeline kicks in */}
          {isDownloaded && !processingInfo && !hasQuestions && (
            <div className="absolute inset-0 bg-black/20 flex items-center justify-center">
              <div className="w-10 h-10 rounded-full bg-green-500/90 flex items-center justify-center">
                <Checkmark className="w-5 h-5 text-white" strokeWidth={2.5} />
              </div>
            </div>
          )}
          {isAssigned && (
            <button
              onClick={onWatch}
              className="absolute inset-0 flex items-center justify-center bg-black/20 hover:bg-black/40 transition-colors group"
            >
              <div className="w-14 h-14 rounded-full bg-white/90 flex items-center justify-center shadow-lg group-hover:scale-105 transition-transform">
                <svg
                  className="w-6 h-6 text-pink-500 ml-1"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="M8 5v14l11-7z" />
                </svg>
              </div>
            </button>
          )}
        </div>

        <div className="p-4 flex flex-col gap-3">
          <div className="flex flex-col gap-1">
            <p className="text-sm font-semibold text-gray-800 line-clamp-2 leading-snug">
              {video.title}
            </p>
            {video.uploader && (
              <p className="text-xs text-gray-400">{video.uploader}</p>
            )}
          </div>

          {/* Status badge — hide while processing is active */}
          {!isAssigned && status && !processingInfo && (
            <VideoStatusBadge status={status} />
          )}

          {/* Active pipeline stage block with animated dot and progress bar */}
          {!isAssigned && stageConfig && (
            <div
              className={`flex flex-col gap-2 rounded-xl px-3 py-2.5 border ${stageConfig.bg}`}
            >
              <div className="flex items-center gap-2">
                <span
                  className={`w-1.5 h-1.5 rounded-full animate-pulse ${stageConfig.dot}`}
                />
                <span className={`text-xs font-medium ${stageConfig.color}`}>
                  {stageConfig.label}
                </span>
              </div>
              {stage === "generating_questions" && progress?.total > 0 && (
                <div className="flex flex-col gap-1">
                  <div className="h-1 w-full rounded-full bg-amber-100">
                    <div
                      className="h-1 rounded-full bg-amber-400 transition-all duration-500"
                      style={{
                        width: `${(progress.current / progress.total) * 100}%`,
                      }}
                    />
                  </div>
                  <p className="text-xs text-amber-400">
                    Segment {progress.current} of {progress.total}
                  </p>
                </div>
              )}
            </div>
          )}

          {/* Pipeline checklist — only shown once downloaded */}
          {!isAssigned && isDownloaded && (
            <PipelineSteps stage={stage} hasQuestions={hasQuestions} />
          )}

          {!isAssigned && hasQuestions && (
            <button
              onClick={onViewQuestions}
              className="w-full py-2 rounded-xl bg-pink-50 border border-pink-200 text-pink-500 text-xs font-medium hover:bg-pink-100 transition-colors"
            >
              View Questions ✨
            </button>
          )}

          {isAssigned && (
            <button
              onClick={onWatch}
              className="w-full py-2.5 rounded-xl bg-gradient-to-r from-pink-400 to-violet-400 text-white text-sm font-semibold hover:opacity-90 transition-opacity"
            >
              ▶ Watch & Learn
            </button>
          )}

          {isRecommendation && !status && !isAssigned && (
            <p className="text-xs text-gray-300 text-center italic">
              Say "download this" to save
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

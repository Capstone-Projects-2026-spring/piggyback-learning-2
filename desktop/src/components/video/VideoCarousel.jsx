import VideoCard from "./VideoCard.jsx";

function ChevronButton({ onClick, disabled, direction }) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className="w-10 h-10 rounded-full bg-white border border-gray-100 shadow-sm flex items-center justify-center text-gray-400 disabled:opacity-30 hover:text-gray-600 transition-colors"
    >
      <svg
        className="w-4 h-4"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d={direction === "left" ? "M15 19l-7-7 7-7" : "M9 5l7 7-7 7"}
        />
      </svg>
    </button>
  );
}

export default function VideoCarousel({
  videos,
  currentIndex,
  onGoTo,
  ytLoading,
  statuses,
  processing,
  questions,
  mode,
  onViewQuestions,
  onWatch,
}) {
  return (
    <div className="flex flex-col gap-5">
      <div className="overflow-hidden">
        <div
          className="flex transition-transform duration-300 ease-in-out"
          style={{ transform: `translateX(-${currentIndex * 100}%)` }}
        >
          {videos.map((video) => (
            <VideoCard
              key={video.video_id}
              video={video}
              status={statuses[video.video_id]}
              processingInfo={processing[video.video_id]}
              hasQuestions={!!questions[video.video_id]}
              isRecommendation={mode === "recommendations"}
              isAssigned={mode === "my_videos"}
              onViewQuestions={() => onViewQuestions(video.video_id)}
              onWatch={() => onWatch(video.video_id)}
            />
          ))}
        </div>
      </div>

      <div className="flex items-center justify-between px-5">
        <ChevronButton
          onClick={() => onGoTo(currentIndex - 1)}
          disabled={currentIndex === 0}
          direction="left"
        />
        <div className="flex items-center gap-1.5">
          {videos.map((_, i) => (
            <button
              key={i}
              onClick={() => onGoTo(i)}
              className={`h-1.5 rounded-full transition-all duration-200 ${
                i === currentIndex ? "bg-pink-400 w-4" : "bg-gray-200 w-1.5"
              }`}
            />
          ))}
          {ytLoading && (
            <span className="w-1.5 h-1.5 rounded-full bg-gray-200 animate-pulse" />
          )}
        </div>
        <ChevronButton
          onClick={() => onGoTo(currentIndex + 1)}
          disabled={currentIndex === videos.length - 1}
          direction="right"
        />
      </div>

      <p className="text-center text-xs text-gray-400">
        {currentIndex + 1} of {videos.length}
        {ytLoading ? " · finding more…" : ""}
      </p>
    </div>
  );
}

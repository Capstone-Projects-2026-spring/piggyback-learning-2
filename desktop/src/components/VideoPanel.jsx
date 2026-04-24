import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { commandBus } from "../lib/stt/commandBus.js";
import QuestionsModal from "./QuestionsModal.jsx";
import WatchVideoPanel from "./WatchVideoPanel.jsx";

function normalizeRec(v) {
  return {
    video_id: v.id,
    title: v.title ?? "Untitled",
    thumbnail:
      v.thumbnail_url ?? `https://i.ytimg.com/vi/${v.id}/mqdefault.jpg`,
    duration: v.duration_seconds ?? null,
    uploader:
      v.score > 0
        ? `${v.score} tag match${v.score !== 1 ? "es" : ""}`
        : "YouTube",
    isYoutube: !v.score || v.score === 0,
  };
}

function normalizeAssigned(v) {
  return {
    video_id: v.id,
    title: v.title ?? "Untitled",
    thumbnail:
      v.thumbnail_url ?? `https://i.ytimg.com/vi/${v.id}/mqdefault.jpg`,
    duration: v.duration_seconds ?? null,
    uploader: "",
    isYoutube: false,
    isAssigned: true,
  };
}

export default function VideoPanel({ onClose, role, initialMyVideos }) {
  const [mode, setMode] = useState(() =>
    initialMyVideos ? "my_videos" : "search",
  );
  const [videos, setVideos] = useState(() =>
    initialMyVideos
      ? (initialMyVideos.videos ?? []).map(normalizeAssigned)
      : [],
  );
  const [loading, setLoading] = useState(() => !initialMyVideos);
  const [query, setQuery] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [kidName, setKidName] = useState(null);
  const [recTags, setRecTags] = useState([]);
  const [ytLoading, setYtLoading] = useState(false);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [statuses, setStatuses] = useState({});
  const [processing, setProcessing] = useState({});
  const [questions, setQuestions] = useState({});
  const [selectedVideoId, setSelectedVideoId] = useState(null);
  const [watchingVideoId, setWatchingVideoId] = useState(null);

  const currentVideoIdRef = useRef(null);
  const modeRef = useRef("search");
  const currentIndexRef = useRef(0);

  useEffect(() => {
    currentVideoIdRef.current = videos[currentIndex]?.video_id ?? null;
    currentIndexRef.current = currentIndex;
  }, [currentIndex, videos]);

  useEffect(() => {
    modeRef.current = mode;
  }, [mode]);

  // Search status
  useEffect(() => {
    let unlisten;
    listen("peppa://search-status", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      if (data.status === "searching" && modeRef.current === "search") {
        setLoading(true);
        setSearchQuery(data.query);
        setVideos([]);
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  // Search results
  useEffect(() => {
    let unlisten;
    listen("peppa://search-results", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      if (modeRef.current === "recommendations") {
        setVideos((current) => {
          const existingIds = new Set(current.map((v) => v.video_id));
          const incoming = (data.results ?? []).filter(
            (v) => !existingIds.has(v.video_id),
          );
          return [...current, ...incoming].slice(0, 10);
        });
        setYtLoading(false);
      } else {
        setVideos(data.results ?? []);
        setQuery(data.query ?? "");
        setSearchQuery("");
        setCurrentIndex(0);
        setLoading(false);
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  // Recommendations
  useEffect(() => {
    let unlisten;
    listen("peppa://recommendations", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      const { kid_name, tags, recommendations } = data;
      setMode("recommendations");
      modeRef.current = "recommendations";
      setKidName(kid_name);
      setRecTags(tags ?? []);
      setCurrentIndex(0);
      setLoading(false);
      setYtLoading(true);
      setVideos((recommendations ?? []).map(normalizeRec));
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  // Download status
  useEffect(() => {
    let unlisten;
    listen("peppa://video-status", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      setStatuses((s) => ({ ...s, [data.video_id]: data.status }));
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  // Processing stages
  useEffect(() => {
    let unlisten;
    listen("peppa://processing-status", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      setProcessing((p) => ({
        ...p,
        [data.video_id]: { stage: data.stage, progress: data.progress },
      }));
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  // Questions ready
  useEffect(() => {
    let unlisten;
    listen("peppa://questions-ready", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      setQuestions((q) => ({ ...q, [data.video_id]: data.segments }));
      setProcessing((p) => {
        const next = { ...p };
        delete next[data.video_id];
        return next;
      });
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  useEffect(() => {
    let unlisten;
    listen("peppa://watch-video", () => {
      const videoId = currentVideoIdRef.current;
      if (videoId) setWatchingVideoId(videoId);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  // Voice: download current card (parent only)
  useEffect(() => {
    const off = commandBus.on("download_video", async () => {
      if (role === "kid") return;
      const videoId = currentVideoIdRef.current;
      if (!videoId) return;
      setStatuses((s) =>
        s[videoId] === "downloading" ? s : { ...s, [videoId]: "downloading" },
      );
      try {
        await invoke("download_video_command", { videoId });
      } catch (e) {
        console.error("[VideoPanel] invoke failed:", e);
        setStatuses((s) => ({ ...s, [videoId]: "error" }));
      }
    });
    return off;
  }, [role]);

  const goTo = (i) =>
    setCurrentIndex(Math.max(0, Math.min(i, videos.length - 1)));

  const headerTitle =
    mode === "my_videos"
      ? "My Videos"
      : mode === "recommendations"
        ? `For ${kidName ?? "…"}`
        : searchQuery
          ? `Searching "${searchQuery}"…`
          : query
            ? `"${query}"`
            : "Videos";

  const headerSub =
    mode === "my_videos"
      ? `Say "watch this" to start`
      : mode === "recommendations" && recTags.length > 0
        ? recTags.join(", ")
        : null;

  return (
    <>
      <div className="fixed inset-0 z-40 flex flex-col bg-gradient-to-b from-pink-50 to-white">
        {/* Header */}
        <div className="flex items-center justify-between px-5 pt-8 pb-4">
          <div className="flex-1 min-w-0">
            <h2 className="text-lg font-bold text-gray-800 truncate">
              {headerTitle}
            </h2>
            <p className="text-xs text-gray-400 mt-0.5">
              {headerSub ?? (
                <>
                  Say{" "}
                  <span className="text-pink-400 font-medium">
                    "download this"
                  </span>
                  {" · "}
                  <span className="text-pink-400 font-medium">
                    "search for …"
                  </span>
                </>
              )}
            </p>
          </div>

          <div className="flex items-center gap-2 ml-3">
            {ytLoading && (
              <div className="flex items-center gap-1.5 text-gray-400 text-xs">
                <span className="w-2 h-2 rounded-full bg-red-400 animate-pulse" />
                YouTube…
              </div>
            )}
            <button
              onClick={onClose}
              className="w-9 h-9 rounded-full bg-white border border-gray-100 shadow-sm flex items-center justify-center text-gray-400 hover:text-gray-600 transition-colors"
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
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 flex flex-col justify-center overflow-hidden">
          {loading ? (
            <div className="flex flex-col items-center gap-4 px-8">
              <div className="w-10 h-10 rounded-full border-2 border-pink-200 border-t-pink-400 animate-spin" />
              {searchQuery ? (
                <>
                  <p className="text-sm font-medium text-gray-500 text-center">
                    Searching for
                  </p>
                  <p className="text-lg font-bold text-pink-400 text-center">
                    "{searchQuery}"
                  </p>
                  <p className="text-xs text-gray-400 text-center">
                    This may take a few seconds…
                  </p>
                </>
              ) : (
                <p className="text-xs text-gray-400">Loading…</p>
              )}
            </div>
          ) : videos.length === 0 && !ytLoading ? (
            <div className="flex flex-col items-center gap-3 px-8">
              {mode === "my_videos" ? (
                <>
                  <p className="text-2xl">🎬</p>
                  <p className="text-sm text-gray-400 text-center">
                    No videos assigned yet.
                  </p>
                  <p className="text-xs text-gray-300 text-center italic">
                    Ask a parent to assign some videos for you
                  </p>
                </>
              ) : mode === "recommendations" ? (
                <>
                  <p className="text-2xl">🎯</p>
                  <p className="text-sm text-gray-400 text-center">
                    No videos found for{" "}
                    <span className="text-pink-400 font-medium">{kidName}</span>{" "}
                    yet.
                  </p>
                  <p className="text-xs text-gray-300 text-center italic">
                    Try adding more interests first
                  </p>
                </>
              ) : (
                <p className="text-sm text-gray-400 text-center">
                  Say{" "}
                  <span className="text-pink-400 font-medium">
                    "search for spiderman"
                  </span>{" "}
                  to find videos
                </p>
              )}
            </div>
          ) : (
            <div className="flex flex-col gap-5">
              <div className="overflow-hidden">
                <div
                  className="flex transition-transform duration-300 ease-in-out"
                  style={{ transform: `translateX(-${currentIndex * 100}%)` }}
                >
                  {videos.map((video, i) => (
                    <VideoCard
                      key={video.video_id}
                      video={video}
                      status={statuses[video.video_id]}
                      processingInfo={processing[video.video_id]}
                      hasQuestions={!!questions[video.video_id]}
                      isRecommendation={mode === "recommendations"}
                      isAssigned={mode === "my_videos"}
                      onViewQuestions={() => setSelectedVideoId(video.video_id)}
                      onWatch={() => setWatchingVideoId(video.video_id)}
                    />
                  ))}
                </div>
              </div>

              {/* Prev / Next */}
              <div className="flex items-center justify-between px-5">
                <button
                  onClick={() => goTo(currentIndex - 1)}
                  disabled={currentIndex === 0}
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
                      d="M15 19l-7-7 7-7"
                    />
                  </svg>
                </button>

                <div className="flex items-center gap-1.5">
                  {videos.map((_, i) => (
                    <button
                      key={i}
                      onClick={() => goTo(i)}
                      className={`h-1.5 rounded-full transition-all duration-200 ${
                        i === currentIndex
                          ? "bg-pink-400 w-4"
                          : "bg-gray-200 w-1.5"
                      }`}
                    />
                  ))}
                  {ytLoading && (
                    <span className="w-1.5 h-1.5 rounded-full bg-gray-200 animate-pulse" />
                  )}
                </div>

                <button
                  onClick={() => goTo(currentIndex + 1)}
                  disabled={currentIndex === videos.length - 1}
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
                      d="M9 5l7 7-7 7"
                    />
                  </svg>
                </button>
              </div>

              <p className="text-center text-xs text-gray-400">
                {currentIndex + 1} of {videos.length}
                {ytLoading ? " · finding more…" : ""}
              </p>
            </div>
          )}
        </div>
      </div>

      {selectedVideoId && questions[selectedVideoId] && (
        <QuestionsModal
          videoId={selectedVideoId}
          segments={questions[selectedVideoId]}
          onClose={() => setSelectedVideoId(null)}
        />
      )}

      {watchingVideoId && (
        <WatchVideoPanel
          videoId={watchingVideoId}
          onClose={() => setWatchingVideoId(null)}
        />
      )}
    </>
  );
}

// ── Stage config ──────────────────────────────────────────────────────────────

const STAGE_CONFIG = {
  tagging: {
    label: "Analysing content…",
    color: "text-violet-500",
    bg: "bg-violet-50 border-violet-100",
    dot: "bg-violet-400",
  },
  extracting_frames: {
    label: "Extracting frames…",
    color: "text-blue-500",
    bg: "bg-blue-50 border-blue-100",
    dot: "bg-blue-400",
  },
  generating_questions: {
    label: "Generating questions…",
    color: "text-amber-500",
    bg: "bg-amber-50 border-amber-100",
    dot: "bg-amber-400",
  },
};

function VideoCard({
  video,
  status,
  processingInfo,
  hasQuestions,
  isRecommendation,
  isAssigned,
  onViewQuestions,
  onWatch,
}) {
  const duration = video.duration
    ? `${Math.floor(video.duration / 60)}:${String(video.duration % 60).padStart(2, "0")}`
    : null;

  const stage = processingInfo?.stage;
  const stageConfig = stage ? STAGE_CONFIG[stage] : null;
  const progress = processingInfo?.progress;

  return (
    <div className="min-w-full px-5">
      <div className="rounded-2xl overflow-hidden bg-white border border-gray-100 shadow-sm">
        {/* Thumbnail */}
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
          {(status === "done" || status === "already_exists") &&
            !processingInfo &&
            !hasQuestions && (
              <div className="absolute inset-0 bg-black/20 flex items-center justify-center">
                <div className="w-10 h-10 rounded-full bg-green-500/90 flex items-center justify-center">
                  <svg
                    className="w-5 h-5 text-white"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2.5}
                      d="M5 13l4 4L19 7"
                    />
                  </svg>
                </div>
              </div>
            )}
          {/* Watch button overlay for assigned videos */}
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

          {/* Parent view: status + pipeline */}
          {!isAssigned && status && !processingInfo && (
            <StatusBadge status={status} />
          )}

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

          {!isAssigned &&
            (status === "done" || status === "already_exists") && (
              <PipelineSteps stage={stage} hasQuestions={hasQuestions} />
            )}

          {/* Parent: view questions button */}
          {!isAssigned && hasQuestions && (
            <button
              onClick={onViewQuestions}
              className="w-full py-2 rounded-xl bg-pink-50 border border-pink-200 text-pink-500 text-xs font-medium hover:bg-pink-100 transition-colors"
            >
              View Questions ✨
            </button>
          )}

          {/* Kid: watch button */}
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

function PipelineSteps({ stage, hasQuestions }) {
  const steps = [
    { key: "tagging", label: "Content tagged" },
    { key: "extracting_frames", label: "Frames extracted" },
    { key: "generating_questions", label: "Questions generated" },
  ];
  const stageOrder = ["tagging", "extracting_frames", "generating_questions"];
  const currentIdx = stage ? stageOrder.indexOf(stage) : -1;

  return (
    <div className="flex flex-col gap-1.5">
      {steps.map((step, i) => {
        const done = hasQuestions || currentIdx > i;
        const active = currentIdx === i;
        return (
          <div key={step.key} className="flex items-center gap-2">
            <div
              className={`w-4 h-4 rounded-full flex items-center justify-center flex-shrink-0 ${
                done
                  ? "bg-green-400"
                  : active
                    ? "bg-amber-400 animate-pulse"
                    : "bg-gray-100"
              }`}
            >
              {done ? (
                <svg
                  className="w-2.5 h-2.5 text-white"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={3}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              ) : (
                <span className="w-1.5 h-1.5 rounded-full bg-gray-300" />
              )}
            </div>
            <span
              className={`text-xs ${
                done
                  ? "text-green-600"
                  : active
                    ? "text-amber-500 font-medium"
                    : "text-gray-300"
              }`}
            >
              {step.label}
            </span>
          </div>
        );
      })}
    </div>
  );
}

function StatusBadge({ status }) {
  const map = {
    downloading: {
      bg: "bg-blue-50 border-blue-100 text-blue-500",
      label: "Downloading…",
    },
    done: { bg: "bg-green-50 border-green-100 text-green-600", label: "Saved" },
    already_exists: {
      bg: "bg-gray-50 border-gray-100 text-gray-500",
      label: "Already saved",
    },
    error: { bg: "bg-red-50 border-red-100 text-red-500", label: "Failed" },
  };
  const s = map[status];
  if (!s) return null;
  return (
    <span
      className={`self-start text-xs px-2 py-0.5 rounded-full border ${s.bg}`}
    >
      {s.label}
    </span>
  );
}

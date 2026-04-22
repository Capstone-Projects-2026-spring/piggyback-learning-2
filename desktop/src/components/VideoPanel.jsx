import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { commandBus } from "../lib/stt/commandBus.js";

export default function VideoPanel({ onClose }) {
  const [videos, setVideos] = useState([]);
  const [loading, setLoading] = useState(false);
  const [query, setQuery] = useState("");
  const [currentIndex, setCurrentIndex] = useState(0);
  const [statuses, setStatuses] = useState({});
  const currentVideoIdRef = useRef(null);

  // Keep ref in sync
  useEffect(() => {
    currentVideoIdRef.current = videos[currentIndex]?.video_id ?? null;
  }, [currentIndex, videos]);

  // Listen for search results from Rust
  useEffect(() => {
    let unlisten;
    listen("peppa://search-results", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      console.log("[VideoPanel] results:", data);
      setVideos(data.results ?? []);
      setQuery(data.query ?? "");
      setCurrentIndex(0);
      setLoading(false);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  // Listen for download_video voice intent
  useEffect(() => {
    const off = commandBus.on("download_video", async () => {
      const videoId = currentVideoIdRef.current;
      if (!videoId) return;
      setStatuses((s) => {
        if (s[videoId] === "downloading") return s;
        return { ...s, [videoId]: "downloading" };
      });
      try {
        await invoke("download_video_command", { videoId });
      } catch (e) {
        console.error("[VideoPanel] invoke failed:", e);
        setStatuses((s) => ({ ...s, [videoId]: "error" }));
      }
    });
    return off;
  }, []);

  // Listen for download status events from Rust
  useEffect(() => {
    let unlisten;
    listen("peppa://video-status", ({ payload }) => {
      const { video_id, status } = payload;
      setStatuses((s) => ({ ...s, [video_id]: status }));
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  const goTo = (i) =>
    setCurrentIndex(Math.max(0, Math.min(i, videos.length - 1)));

  return (
    <div className="fixed inset-0 z-40 flex flex-col bg-gradient-to-b from-pink-50 to-white">
      {/* Header */}
      <div className="flex items-center justify-between px-5 pt-8 pb-4">
        <div>
          <h2 className="text-lg font-bold text-gray-800">
            {query ? `"${query}"` : "Videos"}
          </h2>
          <p className="text-xs text-gray-400 mt-0.5">
            Say{" "}
            <span className="text-pink-400 font-medium">"download this"</span>{" "}
            to save
            {" · "}
            <span className="text-pink-400 font-medium">"search for …"</span> to
            search
          </p>
        </div>
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

      {/* Content */}
      <div className="flex-1 flex flex-col justify-center overflow-hidden">
        {loading ? (
          <div className="flex flex-col items-center gap-3">
            <div className="w-8 h-8 rounded-full border-2 border-pink-200 border-t-pink-400 animate-spin" />
            <p className="text-xs text-gray-400">Searching…</p>
          </div>
        ) : videos.length === 0 ? (
          <div className="flex flex-col items-center gap-3 px-8">
            <p className="text-sm text-gray-400 text-center">
              Say{" "}
              <span className="text-pink-400 font-medium">
                "search for spiderman"
              </span>{" "}
              to find videos
            </p>
          </div>
        ) : (
          <div className="flex flex-col gap-5">
            {/* Sliding cards */}
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
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

function VideoCard({ video, status }) {
  const duration = video.duration
    ? `${Math.floor(video.duration / 60)}:${String(video.duration % 60).padStart(2, "0")}`
    : null;

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
          {status === "downloading" && (
            <div className="absolute inset-0 bg-black/40 flex items-center justify-center">
              <div className="w-8 h-8 rounded-full border-2 border-white/40 border-t-white animate-spin" />
            </div>
          )}
          {(status === "done" || status === "already_exists") && (
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
        </div>
        <div className="p-4 flex flex-col gap-2">
          <p className="text-sm font-semibold text-gray-800 line-clamp-2 leading-snug">
            {video.title}
          </p>
          <p className="text-xs text-gray-400">{video.uploader}</p>
          {status && <StatusBadge status={status} />}
        </div>
      </div>
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

import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { commandBus } from "@/lib";
import QuestionModal from "../questions/QuestionModal.jsx";
import WatchVideoPanel from "../watch/WatchVideoPanel.jsx";
import VideoCarousel from "./VideoCarousel.jsx";
import VideoPanelEmpty from "./VideoPanelEmpty.jsx";
import CloseButton from "../ui/CloseButton.jsx";
import { normalizeRec, normalizeAssigned } from "@/utils";
import { useTauriListener } from "@/hooks";

export default function VideoPanel({ onClose, role }) {
  const [mode, setMode] = useState("search");
  const [videos, setVideos] = useState([]);
  const [loading, setLoading] = useState(false);
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

  useEffect(() => {
    currentVideoIdRef.current = videos[currentIndex]?.video_id ?? null;
  }, [currentIndex, videos]);

  const setModeSync = (m) => {
    modeRef.current = m;
    setMode(m);
  };

  useTauriListener("orb://my-videos", (data) => {
    setModeSync("my_videos");
    setVideos((data.videos ?? []).map(normalizeAssigned));
    setLoading(false);
    setSearchQuery("");
    setQuery("");
    setCurrentIndex(0);
    setYtLoading(false);
  });

  useTauriListener("orb://search-status", (data) => {
    if (data.status === "searching") {
      setModeSync("search");
      setLoading(true);
      setSearchQuery(data.query);
      setVideos([]);
      setCurrentIndex(0);
    }
  });

  useTauriListener("orb://search-results", (data) => {
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
  });

  useTauriListener("orb://recommendations", (data) => {
    const { kid_name, tags, recommendations } = data;
    setModeSync("recommendations");
    setKidName(kid_name);
    setRecTags(tags ?? []);
    setCurrentIndex(0);
    setLoading(false);
    setYtLoading(true);
    setVideos((recommendations ?? []).map(normalizeRec));
  });

  useTauriListener("orb://video-status", (data) => {
    setStatuses((s) => ({ ...s, [data.video_id]: data.status }));
  });

  useTauriListener("orb://processing-status", (data) => {
    setProcessing((p) => ({
      ...p,
      [data.video_id]: { stage: data.stage, progress: data.progress },
    }));
  });

  useTauriListener("orb://questions-ready", (data) => {
    setQuestions((q) => ({ ...q, [data.video_id]: data.segments }));
    setProcessing((p) => {
      const next = { ...p };
      delete next[data.video_id];
      return next;
    });
  });

  useTauriListener("orb://watch-video", () => {
    const videoId = currentVideoIdRef.current;
    if (videoId) setWatchingVideoId(videoId);
  });

  useEffect(() => {
    return commandBus.on("download_video", async () => {
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

  const isEmpty = videos.length === 0 && !ytLoading;

  return (
    <>
      <div className="fixed inset-0 z-40 flex flex-col bg-gradient-to-b from-pink-50 to-white">
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
            <CloseButton
              onClick={onClose}
              className="w-9 h-9 bg-white border border-gray-100 shadow-sm text-gray-400 hover:text-gray-600"
            />
          </div>
        </div>

        <div className="flex-1 flex flex-col justify-center overflow-hidden">
          {loading ? (
            <VideoPanelEmpty
              mode={mode}
              kidName={kidName}
              searchQuery={searchQuery}
            />
          ) : isEmpty ? (
            <VideoPanelEmpty mode={mode} kidName={kidName} searchQuery={null} />
          ) : (
            <VideoCarousel
              videos={videos}
              currentIndex={currentIndex}
              onGoTo={goTo}
              ytLoading={ytLoading}
              statuses={statuses}
              processing={processing}
              questions={questions}
              mode={mode}
              onViewQuestions={setSelectedVideoId}
              onWatch={setWatchingVideoId}
            />
          )}
        </div>
      </div>

      {selectedVideoId && questions[selectedVideoId] && (
        <QuestionModal
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

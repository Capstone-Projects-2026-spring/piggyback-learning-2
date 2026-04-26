export const STAGE_CONFIG = {
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

export const STATUS_MAP = {
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

export const PIPELINE_STEPS = [
  { key: "tagging", label: "Content tagged" },
  { key: "extracting_frames", label: "Frames extracted" },
  { key: "generating_questions", label: "Questions generated" },
];

export function normalizeRec(v) {
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

export function normalizeAssigned(v) {
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

export function formatDuration(seconds) {
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}

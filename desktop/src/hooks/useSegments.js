import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useSegments(videoId) {
  const [segments, setSegments] = useState([]);
  const [videoPath, setVideoPath] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  // Written during render so the ref is always in sync without a separate effect.
  const segmentsRef = useRef(segments);
  segmentsRef.current = segments;

  useEffect(() => {
    if (!videoId) return;

    setLoading(true);
    setError(null);

    invoke("get_segments", { videoId })
      .then((data) => {
        const segs = data ?? [];
        console.log(
          "[useSegments] loaded",
          segs.length,
          "segments for",
          videoId,
        );
        setSegments(segs);
        setVideoPath(segs[0]?.local_video_path ?? null);
      })
      .catch((e) => {
        console.error("[useSegments] failed:", e);
        setError(e);
      })
      .finally(() => setLoading(false));
  }, [videoId]);

  return { segments, segmentsRef, videoPath, loading, error };
}

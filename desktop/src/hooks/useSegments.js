import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useSegments(videoId) {
  const [segments, setSegments] = useState([]);
  const [videoPath, setVideoPath] = useState(null);
  const segmentsRef = useRef([]);

  useEffect(() => {
    segmentsRef.current = segments;
  }, [segments]);

  useEffect(() => {
    if (!videoId) return;
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
        // All segments share the same path — grab from first
        if (segs.length > 0 && segs[0].local_video_path) {
          setVideoPath(segs[0].local_video_path);
        }
      })
      .catch((e) => console.error("[useSegments] failed:", e));
  }, [videoId]);

  return { segments, segmentsRef, videoPath };
}

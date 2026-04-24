import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useSegments(videoId) {
  const [segments, setSegments] = useState([]);
  const segmentsRef = useRef([]);

  useEffect(() => {
    segmentsRef.current = segments;
  }, [segments]);

  useEffect(() => {
    if (!videoId) return;
    invoke("get_segments", { videoId })
      .then((data) => {
        console.log(
          "[useSegments] loaded",
          data?.length,
          "segments for",
          videoId,
        );
        setSegments(data ?? []);
      })
      .catch((e) => console.error("[useSegments] failed:", e));
  }, [videoId]);

  return { segments, segmentsRef };
}

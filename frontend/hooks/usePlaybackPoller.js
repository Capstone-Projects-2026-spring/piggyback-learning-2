import { useEffect, useRef } from "react";

export function usePlaybackPoller({
  playerRef,
  segmentsRef,
  segmentIndexRef,
  currentQuestionRef,
  onSegmentEnd,
  onTick,
}) {
  // Track the exact playback instance so it doesn't double-fire
  const lastTriggeredIdxRef = useRef(-1);

  useEffect(() => {
    const interval = setInterval(() => {
      const player = playerRef.current;
      const segs = segmentsRef.current;
      const idx = segmentIndexRef.current;

      if (!player || segs.length === 0) return;
      if (idx >= segs.length) {
        clearInterval(interval);
        return;
      }

      const segment = segs[idx];
      const currentTime = player.getCurrentTime();

      if (currentTime === undefined || currentTime === 0) return;

      if (onTick) onTick(currentTime, segment);

      const end = segment.end_seconds ?? 0;

      // THE BULLETPROOF FIX: 
      // If the video time is safely *before* the end of the segment (meaning it successfully rewound),
      // we remove the lock so the question can pop up again later.
      if (currentTime < end - 0.5) {
        lastTriggeredIdxRef.current = -1;
      }

      // Trigger the question ONLY if we haven't already triggered it for this specific replay
      if (
        currentTime >= end &&
        !currentQuestionRef.current &&
        lastTriggeredIdxRef.current !== idx
      ) {
        lastTriggeredIdxRef.current = idx;
        player.pauseVideo();
        onSegmentEnd(segment);
      }
    }, 250);

    return () => clearInterval(interval);
  }, []); // eslint-disable-line react-hooks/exhaustive-deps
}
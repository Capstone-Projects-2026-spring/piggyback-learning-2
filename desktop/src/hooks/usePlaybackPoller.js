import { useEffect, useRef } from "react";

export function usePlaybackPoller({
  playerRef,
  segmentsRef,
  segmentIndexRef,
  currentQuestionRef,
  onSegmentEnd,
  onTick,
}) {
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

      let currentTime;
      try {
        currentTime = player.getCurrentTime();
      } catch {
        return;
      }

      if (currentTime === undefined || currentTime === 0) return;

      if (onTick) onTick(currentTime, segment);

      const end = segment.end_seconds ?? 0;

      if (currentTime < end - 0.5) {
        lastTriggeredIdxRef.current = -1;
      }

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

import { useEffect } from "react";

export function usePlaybackPoller({
  playerRef,
  segmentsRef,
  segmentIndexRef,
  currentQuestionRef,
  onSegmentEnd,
  onTick,
}) {
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
      if (onTick) onTick(currentTime, segment);

      if (currentTime >= segment.end_seconds && !currentQuestionRef.current) {
        player.pauseVideo();
        onSegmentEnd(segment);
      }
    }, 250);

    return () => clearInterval(interval);
  }, []); // eslint-disable-line react-hooks/exhaustive-deps
}

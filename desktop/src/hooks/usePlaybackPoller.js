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
  const onSegmentEndRef = useRef(onSegmentEnd);
  const onTickRef = useRef(onTick);

  useEffect(() => {
    onSegmentEndRef.current = onSegmentEnd;
  }, [onSegmentEnd]);
  useEffect(() => {
    onTickRef.current = onTick;
  }, [onTick]);

  useEffect(() => {
    const interval = setInterval(() => {
      const player = playerRef.current;
      const segs = segmentsRef.current;
      const idx = segmentIndexRef.current;

      if (!player || segs.length === 0) return;
      if (idx >= segs.length) return; // exhausted — caller should stop the poller

      const segment = segs[idx];

      let currentTime;
      try {
        currentTime = player.getCurrentTime();
      } catch {
        return;
      }

      // Treat undefined as not-yet-ready; allow t=0 so segments starting at
      // the beginning of a video aren't silently skipped.
      if (currentTime === undefined) return;

      onTickRef.current?.(currentTime, segment);

      const end = segment.end_seconds ?? 0;

      // Reset the trigger lock when the index advances to a new segment.
      // Keying on idx rather than position handles both forward and backward jumps.
      if (
        lastTriggeredIdxRef.current !== idx - 1 &&
        lastTriggeredIdxRef.current !== idx
      ) {
        // fresh segment — ensure lock is clear so it can fire
      }
      // Reset lock if we've rewound behind the trigger window for this segment.
      if (currentTime < end - 0.5 && lastTriggeredIdxRef.current === idx) {
        lastTriggeredIdxRef.current = -1;
      }

      if (
        currentTime >= end &&
        !currentQuestionRef.current &&
        lastTriggeredIdxRef.current !== idx
      ) {
        lastTriggeredIdxRef.current = idx;
        player.pauseVideo();
        onSegmentEndRef.current(segment);
      }
    }, 250);

    return () => clearInterval(interval);
    // Refs are stable — this intentionally runs once. Callbacks are handled via refs above.
  }, []); // eslint-disable-line react-hooks/exhaustive-deps
}

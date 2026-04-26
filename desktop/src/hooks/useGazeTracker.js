import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export function useGazeTracker({ onLookAway, onReturn, enabled, paused }) {
  const isAwayRef = useRef(false);
  const onLookAwayRef = useRef(onLookAway);
  const onReturnRef = useRef(onReturn);
  const pausedRef = useRef(paused);

  // Keep callback refs current so the listener closure never captures stale callbacks.
  useEffect(() => {
    onLookAwayRef.current = onLookAway;
  }, [onLookAway]);
  useEffect(() => {
    onReturnRef.current = onReturn;
  }, [onReturn]);

  // Sync paused state to Rust and resolve any active away state immediately.
  // pausedRef is updated first so the gaze listener sees the new value in the
  // same tick, before any async Tauri round-trip completes.
  useEffect(() => {
    pausedRef.current = paused;

    if (paused) {
      invoke("gaze_pause").catch(() => {});
      // Treat pause as an implicit return so callers (e.g. video player) resume.
      if (isAwayRef.current) {
        isAwayRef.current = false;
        onReturnRef.current?.();
      }
    } else {
      invoke("gaze_resume").catch(() => {});
    }
  }, [paused]);

  useEffect(() => {
    if (!enabled) {
      // Pause emission and reset away state so a subsequent re-enable starts clean.
      invoke("gaze_pause").catch(() => {});
      isAwayRef.current = false;
      return;
    }

    // If the component unmounts or `enabled` changes before listen() resolves,
    // this flag lets the .then() immediately call the unlisten fn rather than leaking.
    let cancelled = false;
    let unlisten = null;

    listen("orb://gaze-status", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;

      if (data.status === "away" && !isAwayRef.current && !pausedRef.current) {
        isAwayRef.current = true;
        onLookAwayRef.current?.();
      } else if (data.status === "present" && isAwayRef.current) {
        isAwayRef.current = false;
        onReturnRef.current?.();
      }
    }).then((fn) => {
      if (cancelled) {
        fn(); // cleanup already ran - unlisten immediately
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
      invoke("gaze_pause").catch(() => {});
      isAwayRef.current = false;
    };
  }, [enabled]);
}

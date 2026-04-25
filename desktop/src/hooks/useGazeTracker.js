import { useEffect, useRef, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export function useGazeTracker({ onLookAway, onReturn, enabled, paused }) {
  const isAwayRef = useRef(false);
  const onLookAwayRef = useRef(onLookAway);
  const onReturnRef = useRef(onReturn);
  const pausedRef = useRef(paused);

  useEffect(() => {
    onLookAwayRef.current = onLookAway;
  }, [onLookAway]);
  useEffect(() => {
    onReturnRef.current = onReturn;
  }, [onReturn]);

  // When paused, tell Rust to suppress events and resolve any active away state
  useEffect(() => {
    pausedRef.current = paused;
    if (paused) {
      invoke("gaze_pause").catch(() => {});
      // If we were away, fire return so the video resumes
      if (isAwayRef.current) {
        isAwayRef.current = false;
        onReturnRef.current?.();
      }
    } else {
      invoke("gaze_resume").catch(() => {});
    }
  }, [paused]);

  useEffect(() => {
    if (!enabled) return;

    let unlisten;
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
      unlisten = fn;
    });

    return () => {
      unlisten?.();
      invoke("gaze_pause").catch(() => {}); // stop emitting when component unmounts
    };
  }, [enabled]);
}

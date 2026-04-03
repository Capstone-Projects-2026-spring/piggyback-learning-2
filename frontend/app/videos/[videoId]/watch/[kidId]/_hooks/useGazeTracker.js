import { useEffect, useRef, useCallback } from "react";
import { FaceLandmarker, FilesetResolver } from "@mediapipe/tasks-vision";

const AWAY_THRESHOLD_MS = 2000;
const MODEL_URL =
  "https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task";
const WASM_URL =
  "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@latest/wasm";

export function useGazeTracker({ onLookAway, onReturn, enabled }) {
  const awayTimerRef = useRef(null);
  const isAwayRef = useRef(false);
  const cancelledRef = useRef(false);
  const animFrameRef = useRef(null);
  const lastTimestampRef = useRef(-1); // track last timestamp to ensure strictly increasing

  const clearAwayTimer = useCallback(() => {
    if (awayTimerRef.current) {
      clearTimeout(awayTimerRef.current);
      awayTimerRef.current = null;
    }
  }, []);

  useEffect(() => {
    if (!enabled) return;

    cancelledRef.current = false;
    isAwayRef.current = false;
    lastTimestampRef.current = -1;

    const video = document.createElement("video");
    video.style.cssText =
      "position:fixed;opacity:0;pointer-events:none;width:1px;height:1px;top:0;left:0;";
    video.setAttribute("playsinline", "true");
    video.muted = true;
    document.body.appendChild(video);

    let stream = null;
    let landmarker = null;

    // rAF passes a DOMHighResTimeStamp — use it directly instead of performance.now()
    const tick = (timestamp) => {
      if (cancelledRef.current) return;

      if (!landmarker || video.readyState < 2) {
        animFrameRef.current = requestAnimationFrame(tick);
        return;
      }

      // Skip frame if timestamp hasn't advanced — guarantees strictly increasing
      if (timestamp <= lastTimestampRef.current) {
        animFrameRef.current = requestAnimationFrame(tick);
        return;
      }
      lastTimestampRef.current = timestamp;

      try {
        const results = landmarker.detectForVideo(video, timestamp);
        const faceDetected =
          results.faceLandmarks && results.faceLandmarks.length > 0;

        if (faceDetected) {
          clearAwayTimer();
          if (isAwayRef.current) {
            isAwayRef.current = false;
            onReturn?.();
          }
        } else {
          if (!isAwayRef.current && !awayTimerRef.current) {
            awayTimerRef.current = setTimeout(() => {
              if (!isAwayRef.current && !cancelledRef.current) {
                isAwayRef.current = true;
                onLookAway?.();
              }
            }, AWAY_THRESHOLD_MS);
          }
        }
      } catch (err) {
        // Swallow per-frame errors so one bad frame doesn't kill the loop
        console.warn("Gaze frame error:", err);
      }

      animFrameRef.current = requestAnimationFrame(tick);
    };

    (async () => {
      try {
        stream = await navigator.mediaDevices.getUserMedia({
          video: { width: 320, height: 240, facingMode: "user" },
          audio: false,
        });

        if (cancelledRef.current) {
          stream.getTracks().forEach((t) => t.stop());
          return;
        }

        video.srcObject = stream;
        await video.play();

        const vision = await FilesetResolver.forVisionTasks(WASM_URL);
        if (cancelledRef.current) return;

        landmarker = await FaceLandmarker.createFromOptions(vision, {
          baseOptions: {
            modelAssetPath: MODEL_URL,
            delegate: "GPU",
          },
          runningMode: "VIDEO",
          numFaces: 1,
          minFaceDetectionConfidence: 0.5,
          minFacePresenceConfidence: 0.5,
          minTrackingConfidence: 0.5,
        });

        if (cancelledRef.current) return;

        animFrameRef.current = requestAnimationFrame(tick);
      } catch (err) {
        console.error("Gaze tracker failed to initialize:", err);
      }
    })();

    return () => {
      cancelledRef.current = true;
      clearAwayTimer();
      isAwayRef.current = false;
      cancelAnimationFrame(animFrameRef.current);
      stream?.getTracks().forEach((t) => t.stop());
      try {
        landmarker?.close();
      } catch (_) {}
      video.remove();
    };
  }, [enabled]); // eslint-disable-line react-hooks/exhaustive-deps
}

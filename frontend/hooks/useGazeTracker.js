import { useEffect, useRef, useCallback } from "react";
import { FaceLandmarker, FilesetResolver } from "@mediapipe/tasks-vision";

const AWAY_THRESHOLD_MS = 2000;
const RETURN_CONSECUTIVE_FRAMES = 5; // require 5 solid frames before confirming return
const MODEL_URL =
  "https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task";
const WASM_URL =
  "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@latest/wasm";

export function useGazeTracker({ onLookAway, onReturn, enabled, paused }) {
  const awayTimerRef = useRef(null);
  const isAwayRef = useRef(false);
  const cancelledRef = useRef(false);
  const animFrameRef = useRef(null);
  const lastTimestampRef = useRef(-1);
  const consecutiveFaceFramesRef = useRef(0); // counts consecutive frames with face
  const pausedRef = useRef(paused);
  const onLookAwayRef = useRef(onLookAway);
  const onReturnRef = useRef(onReturn);

  useEffect(() => {
    pausedRef.current = paused;
  }, [paused]);
  useEffect(() => {
    onLookAwayRef.current = onLookAway;
  }, [onLookAway]);
  useEffect(() => {
    onReturnRef.current = onReturn;
  }, [onReturn]);

  useEffect(() => {
    if (!paused) {
      if (awayTimerRef.current) {
        clearTimeout(awayTimerRef.current);
        awayTimerRef.current = null;
      }
      isAwayRef.current = false;
      consecutiveFaceFramesRef.current = 0;
    }
  }, [paused]);

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
    consecutiveFaceFramesRef.current = 0;

    const video = document.createElement("video");
    video.style.cssText =
      "position:fixed;opacity:0;pointer-events:none;width:1px;height:1px;top:0;left:0;";
    video.setAttribute("playsinline", "true");
    video.muted = true;
    document.body.appendChild(video);

    let stream = null;
    let landmarker = null;

    const tick = (timestamp) => {
      if (cancelledRef.current) return;

      if (!landmarker || video.readyState < 2) {
        animFrameRef.current = requestAnimationFrame(tick);
        return;
      }

      if (timestamp <= lastTimestampRef.current) {
        animFrameRef.current = requestAnimationFrame(tick);
        return;
      }
      lastTimestampRef.current = timestamp;

      try {
        const results = landmarker.detectForVideo(video, timestamp);

        if (!pausedRef.current) {
          const faceDetected =
            results.faceLandmarks && results.faceLandmarks.length > 0;

          if (faceDetected) {
            consecutiveFaceFramesRef.current += 1;

            // Cancel any pending away timer — they're looking back
            clearAwayTimer();

            // Confirm return only after N consecutive frames with a face
            // This handles both normal return and coming back from fully out of frame
            if (
              isAwayRef.current &&
              consecutiveFaceFramesRef.current >= RETURN_CONSECUTIVE_FRAMES
            ) {
              isAwayRef.current = false;
              onReturnRef.current?.();
            }
          } else {
            // Reset consecutive counter the moment face is lost
            consecutiveFaceFramesRef.current = 0;

            if (!isAwayRef.current && !awayTimerRef.current) {
              awayTimerRef.current = setTimeout(() => {
                if (!cancelledRef.current) {
                  isAwayRef.current = true;
                  onLookAwayRef.current?.();
                  awayTimerRef.current = null;
                }
              }, AWAY_THRESHOLD_MS);
            }
          }
        }
      } catch (err) {
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
      consecutiveFaceFramesRef.current = 0;
      cancelAnimationFrame(animFrameRef.current);
      stream?.getTracks().forEach((t) => t.stop());
      try {
        landmarker?.close();
      } catch (_) {}
      video.remove();
    };
  }, [enabled]); // eslint-disable-line react-hooks/exhaustive-deps
}

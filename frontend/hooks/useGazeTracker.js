import { useEffect, useRef, useCallback } from "react";
import { FaceLandmarker, FilesetResolver } from "@mediapipe/tasks-vision";

const AWAY_THRESHOLD_MS = 2000;
const RETURN_CONSECUTIVE_FRAMES = 5;
const MODEL_URL =
  "https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task";
const WASM_URL =
  "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@latest/wasm";

export function useGazeTracker({
  onLookAway,
  onReturn,
  enabled,
  paused,
  debug = true,
  thresholdLeftRight = 0.55,
  thresholdUp = 0.55,
  thresholdDown = 0.55,
}) {
  const awayTimerRef = useRef(null);
  const isAwayRef = useRef(false);
  const consecutiveFaceFramesRef = useRef(0);

  const pausedRef = useRef(paused);
  const onLookAwayRef = useRef(onLookAway);
  const onReturnRef = useRef(onReturn);

  useEffect(() => { pausedRef.current = paused; }, [paused]);
  useEffect(() => { onLookAwayRef.current = onLookAway; }, [onLookAway]);
  useEffect(() => { onReturnRef.current = onReturn; }, [onReturn]);

  useEffect(() => {
    if (paused) {
      if (awayTimerRef.current) {
        clearTimeout(awayTimerRef.current);
        awayTimerRef.current = null;
      }
      if (isAwayRef.current) {
        isAwayRef.current = false;
        onReturnRef.current?.();
      }
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

    let isEffectActive = true;
    let localAnimFrame = null;
    let localLandmarker = null;
    let localStream = null;
    let lastVideoTime = -1;
    let lastTimestampMs = -1;

    isAwayRef.current = false;
    consecutiveFaceFramesRef.current = 0;

    const existingDebug = document.getElementById("gaze-debug-ui");
    if (existingDebug) existingDebug.remove();

    const video = document.createElement("video");
    video.setAttribute("playsinline", "true");
    video.muted = true;
    video.style.cssText = "position:absolute; top:-9999px; left:-9999px; width:320px; height:240px; pointer-events:none;";
    document.body.appendChild(video);

    const canvas = document.createElement("canvas");
    canvas.width = 320;
    canvas.height = 240;
    const ctx = canvas.getContext("2d", { willReadFrequently: true });

    let debugContainer = null;
    let debugText = null;

    if (debug) {
      debugContainer = document.createElement("div");
      debugContainer.id = "gaze-debug-ui";
      debugContainer.style.cssText = `
        position: fixed; bottom: 20px; right: 20px; 
        background: rgba(0,0,0,0.85); color: white; 
        padding: 12px; border-radius: 10px; z-index: 999999; 
        font-family: monospace; font-size: 13px; pointer-events: none;
        box-shadow: 0px 4px 12px rgba(0,0,0,0.5);
      `;

      video.style.cssText = "width: 200px; height: 150px; transform: scaleX(-1); border-radius: 6px; display: block; object-fit: cover; background: #222;";
      debugContainer.appendChild(video);

      debugText = document.createElement("div");
      debugText.style.cssText = "margin-top: 10px; line-height: 1.6;";
      debugContainer.appendChild(debugText);
      document.body.appendChild(debugContainer);
      
      debugText.innerHTML = "Initializing AI model...";
    }

    const tick = () => {
      if (!isEffectActive) return;

      if (!localLandmarker || video.readyState < 2 || video.videoWidth === 0) {
        localAnimFrame = requestAnimationFrame(tick);
        return;
      }

      if (video.currentTime !== lastVideoTime) {
        lastVideoTime = video.currentTime;
        
        try {
          if (canvas.width !== video.videoWidth) {
            canvas.width = video.videoWidth;
            canvas.height = video.videoHeight;
          }

          ctx.drawImage(video, 0, 0, canvas.width, canvas.height);

          let nowInMs = Math.round(performance.now());
          if (nowInMs <= lastTimestampMs) {
            nowInMs = lastTimestampMs + 1;
          }
          lastTimestampMs = nowInMs;

          const results = localLandmarker.detectForVideo(canvas, nowInMs);

          if (!pausedRef.current) {
            const faceDetected = results.faceLandmarks && results.faceLandmarks.length > 0;

            let isEyesAway = false;
            let gazeDirection = "Center";
            let lLeft = 0, lRight = 0, lUp = 0, lDown = 0;
            let hasBlendshapes = false;

            // THE FIX: Changed to faceBlendshapes instead of facialBlendshapes
            if (faceDetected && results.faceBlendshapes && results.faceBlendshapes.length > 0) {
              hasBlendshapes = true;
              const blendshapes = results.faceBlendshapes[0].categories;
              const getScore = (name) => blendshapes.find((b) => b.categoryName === name)?.score || 0;

              lLeft = Math.max(getScore("eyeLookOutLeft"), getScore("eyeLookInRight"));
              lRight = Math.max(getScore("eyeLookInLeft"), getScore("eyeLookOutRight"));
              lUp = Math.max(getScore("eyeLookUpLeft"), getScore("eyeLookUpRight"));
              lDown = Math.max(getScore("eyeLookDownLeft"), getScore("eyeLookDownRight"));

              if (lLeft > thresholdLeftRight) { isEyesAway = true; gazeDirection = "Left"; } 
              else if (lRight > thresholdLeftRight) { isEyesAway = true; gazeDirection = "Right"; } 
              else if (lUp > thresholdUp) { isEyesAway = true; gazeDirection = "Up"; } 
              else if (lDown > thresholdDown) { isEyesAway = true; gazeDirection = "Down"; }
            }

            const isDistracted = !faceDetected || isEyesAway;

            if (debug && debugText) {
              const formatRow = (label, val, thresh) => {
                const isOver = val > thresh;
                return `<span style="color:${isOver ? "#ff4444" : "#ffffff"}"><b>${label}:</b> ${val.toFixed(2)} / ${thresh.toFixed(2)}</span><br/>`;
              };

              if (!faceDetected) {
                 debugText.innerHTML = `<b>Status:</b> <span style="color:#ff4444">DISTRACTED (No Face)</span>`;
              } else if (!hasBlendshapes) {
                 debugText.innerHTML = `<b>Status:</b> <span style="color:yellow">WAITING ON AI DATA...</span>`;
              } else {
                debugText.innerHTML = `
                  <b>Status:</b> <span style="color:${isDistracted ? "#ff4444" : "#44ff44"}">${isDistracted ? "DISTRACTED" : "FOCUSED"}</span><br/>
                  <b>Gaze:</b> ${gazeDirection}<br/>
                  <hr style="border: 1px solid #444; margin: 8px 0;" />
                  <span style="color:#aaa; font-size: 11px; text-transform: uppercase;">Current / Threshold</span><br/>
                  ${formatRow("Left", lLeft, thresholdLeftRight)}
                  ${formatRow("Right", lRight, thresholdLeftRight)}
                  ${formatRow("Up", lUp, thresholdUp)}
                  ${formatRow("Down", lDown, thresholdDown)}
                `;
              }
            }

            // Only update logic if we actually successfully pulled the AI data
            if (hasBlendshapes || !faceDetected) {
              if (!isDistracted) {
                consecutiveFaceFramesRef.current += 1;
                clearAwayTimer();

                if (isAwayRef.current && consecutiveFaceFramesRef.current >= RETURN_CONSECUTIVE_FRAMES) {
                  isAwayRef.current = false;
                  onReturnRef.current?.();
                }
              } else {
                consecutiveFaceFramesRef.current = 0;

                if (!isAwayRef.current && !awayTimerRef.current) {
                  awayTimerRef.current = setTimeout(() => {
                    if (isEffectActive) {
                      isAwayRef.current = true;
                      onLookAwayRef.current?.();
                      awayTimerRef.current = null;
                    }
                  }, AWAY_THRESHOLD_MS);
                }
              }
            }
          }
        } catch (err) {
          console.warn("MediaPipe Frame Processing Error:", err);
        }
      }

      localAnimFrame = requestAnimationFrame(tick);
    };

    (async () => {
      try {
        localStream = await navigator.mediaDevices.getUserMedia({
          video: { width: 320, height: 240, facingMode: "user" },
          audio: false,
        });

        if (!isEffectActive) return;

        video.srcObject = localStream;
        await video.play().catch(e => console.error("Video play blocked:", e));

        const vision = await FilesetResolver.forVisionTasks(WASM_URL);
        if (!isEffectActive) return;

        const aiModel = await FaceLandmarker.createFromOptions(vision, {
          baseOptions: {
            modelAssetPath: MODEL_URL,
            delegate: "GPU", 
          },
          runningMode: "VIDEO",
          numFaces: 1,
          outputFaceBlendshapes: true, // THE FIX: Changed from outputFacialBlendshapes
          minFaceDetectionConfidence: 0.5,
          minFacePresenceConfidence: 0.5,
          minTrackingConfidence: 0.5,
        });

        if (!isEffectActive) {
          aiModel.close();
          return;
        }

        localLandmarker = aiModel;
        localAnimFrame = requestAnimationFrame(tick);
      } catch (err) {
        console.error("Gaze tracker initialization failed:", err);
        if (debug && debugText) {
          debugText.innerHTML = `<span style="color:#ff4444">ERROR: Check console.</span>`;
        }
      }
    })();

    return () => {
      isEffectActive = false;
      clearAwayTimer();
      
      if (localAnimFrame) cancelAnimationFrame(localAnimFrame);
      
      if (localStream) {
        localStream.getTracks().forEach((t) => {
          t.stop();
          localStream.removeTrack(t);
        });
      }
      
      try { localLandmarker?.close(); } catch (_) {}
      
      if (debugContainer) debugContainer.remove();
      else video.remove();
      
      canvas.remove();
    };
  }, [enabled, debug, thresholdLeftRight, thresholdUp, thresholdDown, clearAwayTimer]);
}
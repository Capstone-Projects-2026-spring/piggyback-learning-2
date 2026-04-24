import { useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

function floatTo16BitPCM(output, offset, input) {
  for (let i = 0; i < input.length; i++, offset += 2) {
    const s = Math.max(-1, Math.min(1, input[i]));
    output.setInt16(offset, s < 0 ? s * 0x8000 : s * 0x7fff, true);
  }
}

function encodeWAV(samples, sampleRate) {
  const buffer = new ArrayBuffer(44 + samples.length * 2);
  const view = new DataView(buffer);
  const writeString = (offset, str) => {
    for (let i = 0; i < str.length; i++)
      view.setUint8(offset + i, str.charCodeAt(i));
  };
  writeString(0, "RIFF");
  view.setUint32(4, 36 + samples.length * 2, true);
  writeString(8, "WAVE");
  writeString(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  writeString(36, "data");
  view.setUint32(40, samples.length * 2, true);
  floatTo16BitPCM(view, 44, samples);
  return new Blob([buffer], { type: "audio/wav" });
}

function downsampleBuffer(buffer, inputRate, outputRate) {
  if (inputRate === outputRate) return buffer;
  const ratio = inputRate / outputRate;
  const newLength = Math.round(buffer.length / ratio);
  const result = new Float32Array(newLength);
  for (let i = 0; i < newLength; i++) {
    result[i] = buffer[Math.round(i * ratio)];
  }
  return result;
}

const TARGET_SAMPLE_RATE = 16000;
const RECORD_DELAY_MS = 3000;
const RECORD_DURATION_MS = 3000;

export function useAudioRecorder({ onStateChange, onStatusChange, onResult }) {
  const timeoutsRef = useRef([]);

  const clearTimeouts = useCallback(() => {
    timeoutsRef.current.forEach(clearTimeout);
    timeoutsRef.current = [];
  }, []);

  const start = useCallback(
    (segment, extraFields = {}) => {
      clearTimeouts();

      const outerTimeout = setTimeout(async () => {
        onStateChange("waiting");
        onStatusChange("Get ready to answer in 3 seconds…");

        await new Promise((resolve) => {
          const t = setTimeout(resolve, RECORD_DELAY_MS);
          timeoutsRef.current.push(t);
        });

        let stream;
        try {
          stream = await navigator.mediaDevices.getUserMedia({
            audio: { channelCount: 1, sampleRate: TARGET_SAMPLE_RATE },
          });
        } catch (err) {
          console.error("Microphone access denied:", err);
          onStatusChange("Microphone access denied.");
          onStateChange("idle");
          return;
        }

        const audioCtx = new AudioContext({ sampleRate: TARGET_SAMPLE_RATE });
        const source = audioCtx.createMediaStreamSource(stream);
        const processor = audioCtx.createScriptProcessor(4096, 1, 1);
        const chunks = [];

        source.connect(processor);
        processor.connect(audioCtx.destination);
        processor.onaudioprocess = (e) => {
          chunks.push(new Float32Array(e.inputBuffer.getChannelData(0)));
        };

        onStateChange("recording");
        onStatusChange("🎙️ Recording your answer…");

        await new Promise((resolve) => {
          const t = setTimeout(resolve, RECORD_DURATION_MS);
          timeoutsRef.current.push(t);
        });

        processor.disconnect();
        source.disconnect();
        audioCtx.close();
        stream.getTracks().forEach((t) => t.stop());

        const totalLength = chunks.reduce((sum, c) => sum + c.length, 0);
        const merged = new Float32Array(totalLength);
        let offset = 0;
        for (const chunk of chunks) {
          merged.set(chunk, offset);
          offset += chunk.length;
        }

        const downsampled = downsampleBuffer(
          merged,
          audioCtx.sampleRate,
          TARGET_SAMPLE_RATE,
        );
        const wavBlob = encodeWAV(downsampled, TARGET_SAMPLE_RATE);

        onStateChange("analyzing");
        onStatusChange("Analyzing your answer…");

        try {
          const expectedAnswer =
            extraFields.expected_answer_override ??
            segment.questions?.find(
              (x) =>
                x.question === segment.best_question ||
                x.question?.includes(segment.best_question),
            )?.answer;

          const arrayBuffer = await wavBlob.arrayBuffer();
          const uint8 = new Uint8Array(arrayBuffer);
          const base64 = btoa(String.fromCharCode(...uint8));

          const result = await invoke("analyze_answer", {
            audioBase64: base64,
            expectedAnswer: expectedAnswer ?? "",
            kidId: extraFields.kid_id ?? 0,
            videoId: extraFields.video_id ?? "",
            segmentId: extraFields.segment_id ?? 0,
          });

          onResult(result);
        } catch (err) {
          console.error("[useAudioRecorder] analyze failed:", err);
          onStatusChange("Analysis failed. Continuing…");
          onStateChange("idle");
          onResult(null);
        }
      }, 0);

      timeoutsRef.current.push(outerTimeout);
    },
    [clearTimeouts, onStateChange, onStatusChange, onResult],
  );

  useEffect(() => () => clearTimeouts(), [clearTimeouts]);

  return { start, cancel: clearTimeouts };
}

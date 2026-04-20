import { invoke } from "@tauri-apps/api/core";
import { commandBus } from "./commandBus.js";

const CHUNK_MS = 5000;
const SAMPLE_RATE = 16000;

let mediaRecorder = null;
let audioContext = null;
let isRunning = false;

// ── Start / stop ─────────────────────────────────────────────────────────────

export async function startPeppa() {
  if (isRunning) return;
  isRunning = true;

  const stream = await navigator.mediaDevices.getUserMedia({
    audio: {
      channelCount: 1,
      sampleRate: SAMPLE_RATE,
      echoCancellation: true,
      noiseSuppression: true,
    },
  });

  // Use AudioContext to guarantee 16kHz mono WAV output
  audioContext = new AudioContext({ sampleRate: SAMPLE_RATE });
  const source = audioContext.createMediaStreamSource(stream);

  // ScriptProcessor gives us raw PCM we can encode ourselves
  const processor = audioContext.createScriptProcessor(4096, 1, 1);
  let pcmBuffer = [];

  source.connect(processor);
  processor.connect(audioContext.destination);

  processor.onaudioprocess = (e) => {
    const float32 = e.inputBuffer.getChannelData(0);
    // Convert float32 [-1,1] to int16
    for (let i = 0; i < float32.length; i++) {
      const clamped = Math.max(-1, Math.min(1, float32[i]));
      pcmBuffer.push(Math.round(clamped * 32767));
    }
  };

  // Every CHUNK_MS, flush the buffer as a WAV and send to Rust
  const flushInterval = setInterval(async () => {
    if (!isRunning || pcmBuffer.length === 0) return;

    const samples = new Int16Array(pcmBuffer.splice(0));
    const wavBytes = encodePcmToWav(samples, SAMPLE_RATE);

    try {
      const result = await invoke("process_audio", {
        wavBytes: Array.from(new Uint8Array(wavBytes)),
      });

      if (result.wake_detected && result.command) {
        commandBus.dispatch(result.command);
      }

      // Optional: expose partial transcript for live UI feedback
      if (result.transcript) {
        commandBus.dispatchTranscript(result.transcript);
      }
    } catch (err) {
      console.error("[Peppa STT] process_audio error:", err);
    }
  }, CHUNK_MS);

  // Store cleanup refs
  mediaRecorder = { stream, processor, source, flushInterval };
  console.info('[Peppa] Listening for "hey peppa"...');
}

export function stopPeppa() {
  if (!isRunning) return;
  isRunning = false;

  clearInterval(mediaRecorder?.flushInterval);
  mediaRecorder?.processor?.disconnect();
  mediaRecorder?.source?.disconnect();
  mediaRecorder?.stream?.getTracks().forEach((t) => t.stop());
  audioContext?.close();

  mediaRecorder = null;
  audioContext = null;
  console.info("[Peppa] Stopped.");
}

// ── WAV encoder ───────────────────────────────────────────────────────────────
// Produces a minimal valid PCM WAV that hound can parse on the Rust side.

function encodePcmToWav(samples, sampleRate) {
  const byteRate = sampleRate * 2; // 16-bit mono
  const dataLength = samples.length * 2;
  const buffer = new ArrayBuffer(44 + dataLength);
  const view = new DataView(buffer);

  const writeStr = (offset, str) => {
    for (let i = 0; i < str.length; i++)
      view.setUint8(offset + i, str.charCodeAt(i));
  };

  writeStr(0, "RIFF");
  view.setUint32(4, 36 + dataLength, true);
  writeStr(8, "WAVE");
  writeStr(12, "fmt ");
  view.setUint32(16, 16, true); // PCM chunk size
  view.setUint16(20, 1, true); // PCM format
  view.setUint16(22, 1, true); // mono
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, byteRate, true);
  view.setUint16(32, 2, true); // block align
  view.setUint16(34, 16, true); // bits per sample
  writeStr(36, "data");
  view.setUint32(40, dataLength, true);

  let offset = 44;
  for (let i = 0; i < samples.length; i++, offset += 2) {
    view.setInt16(offset, samples[i], true);
  }

  return buffer;
}

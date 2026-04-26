import { listen } from "@tauri-apps/api/event";
import { commandBus } from "./commandBus.js";

let unlistenVoice = null;
let unlistenEnrollment = null;
let unlistenSpeaker = null;

export async function startOrb() {
  if (unlistenVoice) return;

  unlistenEnrollment = await listen("orb://enrollment", ({ payload }) => {
    console.log("[orb] enrollment:", payload.stage, payload);
    commandBus.dispatchEnrollment(payload);
  });

  unlistenSpeaker = await listen("orb://speaker-identified", ({ payload }) => {
    commandBus.dispatchSpeaker(payload);
  });

  unlistenVoice = await listen("orb://voice-result", ({ payload }) => {
    if (payload.transcript) commandBus.dispatchTranscript(payload.transcript);
    if (payload.wake_detected) commandBus.dispatchWake();
    else if (payload.command) commandBus.dispatch(payload.command);
  });

  console.info("[orb] listeners registered");
}

export function stopOrb() {
  unlistenVoice?.();
  unlistenEnrollment?.();
  unlistenSpeaker?.();
  unlistenVoice = null;
  unlistenEnrollment = null;
  unlistenSpeaker = null;
}

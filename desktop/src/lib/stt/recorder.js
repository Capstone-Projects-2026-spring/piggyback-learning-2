import { listen } from "@tauri-apps/api/event";
import { commandBus } from "./commandBus.js";

let unlistenVoice = null;
let unlistenEnrollment = null;

export async function startPeppa() {
  if (unlistenVoice) return;

  // Register enrollment listener first
  unlistenEnrollment = await listen("peppa://enrollment", (event) => {
    let data = event.payload;
    if (typeof data === "string") {
      try {
        data = JSON.parse(data);
      } catch (e) {
        console.error("[recorder] parse failed:", e);
        return;
      }
    }
    console.log("[recorder] enrollment:", data.stage, data);
    commandBus.dispatchEnrollment(data);
  });

  unlistenVoice = await listen("peppa://voice-result", (event) => {
    let result = event.payload;
    if (typeof result === "string") {
      try {
        result = JSON.parse(result);
      } catch (e) {
        console.error("[recorder] parse failed:", e);
        return;
      }
    }
    if (result.transcript) commandBus.dispatchTranscript(result.transcript);
    if (result.wake_detected) {
      commandBus.dispatchWake(!!result.speaker_identified);
    } else if (result.command) {
      commandBus.dispatch(result.command);
    }
  });

  console.info("[recorder] listeners registered");
}

export function stopPeppa() {
  unlistenVoice?.();
  unlistenEnrollment?.();
  unlistenVoice = null;
  unlistenEnrollment = null;
}

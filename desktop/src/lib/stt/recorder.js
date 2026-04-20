import { listen } from "@tauri-apps/api/event";
import { commandBus } from "./commandBus.js";

let unlisten = null;

export async function startPeppa() {
  if (unlisten) return;

  unlisten = await listen("peppa://voice-result", (event) => {
    const result = event.payload;
    console.log("[recorder] voice-result:", JSON.stringify(result));

    if (result.transcript) {
      console.log("[recorder] transcript:", result.transcript);
      commandBus.dispatchTranscript(result.transcript);
    }

    if (result.wake_detected) {
      if (result.command) {
        console.log("[recorder] dispatching command:", result.command);
        if (typeof result.command.intent === "undefined") {
          console.warn(
            "[recorder] command missing 'intent'. Got keys:",
            Object.keys(result.command),
          );
        }
        commandBus.dispatch(result.command);
      } else {
        console.log("[recorder] wake only");
        commandBus.dispatch({
          intent: "wake_only",
          args: null,
          raw: result.transcript,
        });
      }
    } else {
      console.log("[recorder] no wake in this chunk");
    }
  });

  console.info("[recorder] listening for peppa://voice-result events");
}

export function stopPeppa() {
  unlisten?.();
  unlisten = null;
  console.info("[recorder] stopped");
}

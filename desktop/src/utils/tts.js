import { invoke } from "@tauri-apps/api/core";

export function speak(text) {
  if (!text) return;
  // Stop any ongoing speech before starting new one
  invoke("stop_speaking").catch(() => {});
  invoke("speak", { text }).catch((e) =>
    console.error("[tts] speak failed:", e),
  );
}

export function stopSpeaking() {
  invoke("stop_speaking").catch(() => {});
}

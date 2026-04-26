import { invoke } from "@tauri-apps/api/core";

export function speak(text) {
  if (!text) return;
  invoke("speak", { text }).catch((e) =>
    console.error("[tts] speak failed:", e),
  );
}

export function stopSpeaking() {
  invoke("stop_speaking").catch(() => {});
}

const INTENT_MAP = {
  open: ["open", "show", "launch", "go", "start"],
  close: ["close", "hide", "dismiss", "exit", "quit"],
  play: ["play", "watch"],
  stop: ["stop", "pause", "freeze"],
  search: ["search", "find", "look"],
  volume: ["volume", "louder", "quieter", "mute", "unmute"],
  help: ["help", "what", "how"],
  wake_only: [], // wake word with no command body
  chat: [], // catch-all
};

class CommandBus {
  #handlers = new Map();
  #wildcards = new Set();
  #transcriptListeners = new Set();

  /**
   * Subscribe to a resolved command intent.
   * Use '*' to receive every command regardless of intent.
   * Returns an unsubscribe function.
   */
  on(intent, handler) {
    if (intent === "*") {
      this.#wildcards.add(handler);
      return () => this.#wildcards.delete(handler);
    }
    if (!this.#handlers.has(intent)) this.#handlers.set(intent, new Set());
    this.#handlers.get(intent).add(handler);
    return () => this.#handlers.get(intent)?.delete(handler);
  }

  /**
   * Subscribe to raw transcript text (every 5s chunk, wake word or not).
   * Useful for showing a live "heard: ..." indicator in the UI.
   */
  onTranscript(handler) {
    this.#transcriptListeners.add(handler);
    return () => this.#transcriptListeners.delete(handler);
  }

  /** Called by recorder.js with the ResolvedCommand from Rust. */
  dispatch({ intent, args, raw }) {
    const event = { intent, args, raw };
    console.info("[Peppa Bus] ←", event);

    this.#handlers.get(intent)?.forEach((h) => {
      try {
        h(event);
      } catch (e) {
        console.error("[Peppa Bus]", e);
      }
    });
    this.#wildcards.forEach((h) => {
      try {
        h(event);
      } catch (e) {
        console.error("[Peppa Bus]", e);
      }
    });
  }

  /** Called by recorder.js with the raw transcript string. */
  dispatchTranscript(text) {
    this.#transcriptListeners.forEach((h) => {
      try {
        h(text);
      } catch (e) {
        console.error("[Peppa Bus]", e);
      }
    });
  }
}

export const commandBus = new CommandBus();

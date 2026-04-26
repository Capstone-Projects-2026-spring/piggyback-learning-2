class CommandBus {
  #handlers = new Map();
  #wildcards = new Set();
  #transcriptListeners = new Set();
  #speakerListeners = new Set();
  #wakeListeners = new Set();
  #enrollmentListeners = new Set();
  #lastEnrollmentEvent = null;

  // Safe handler invocation — isolates listener errors from each other
  #call(handler, payload) {
    try {
      handler(payload);
    } catch (e) {
      console.error("[commandBus] handler error:", e);
    }
  }

  on(intent, handler) {
    if (intent === "*") {
      this.#wildcards.add(handler);
      return () => this.#wildcards.delete(handler);
    }
    if (!this.#handlers.has(intent)) this.#handlers.set(intent, new Set());
    this.#handlers.get(intent).add(handler);
    return () => this.#handlers.get(intent)?.delete(handler);
  }

  onTranscript(handler) {
    this.#transcriptListeners.add(handler);
    return () => this.#transcriptListeners.delete(handler);
  }

  onWake(handler) {
    this.#wakeListeners.add(handler);
    return () => this.#wakeListeners.delete(handler);
  }

  onSpeaker(handler) {
    this.#speakerListeners.add(handler);
    return () => this.#speakerListeners.delete(handler);
  }

  onEnrollment(handler) {
    this.#enrollmentListeners.add(handler);
    // Replay cached parent enrollment events — these can arrive before the
    // subscriber registers since the overlay mounts after the Tauri event fires.
    // Kid enrollment is always triggered while the overlay is already mounted.
    if (this.#lastEnrollmentEvent?.flow === "parent") {
      this.#call(handler, this.#lastEnrollmentEvent);
    }
    return () => this.#enrollmentListeners.delete(handler);
  }

  dispatch({ intent, args, raw }) {
    const event = { intent, args, raw };
    console.info("[orb] command ←", event);
    this.#handlers.get(intent)?.forEach((h) => this.#call(h, event));
    this.#wildcards.forEach((h) => this.#call(h, event));
  }

  dispatchTranscript(text) {
    this.#transcriptListeners.forEach((h) => this.#call(h, text));
  }

  dispatchWake() {
    console.info("[orb] wake detected");
    this.#wakeListeners.forEach((h) => this.#call(h));
  }

  dispatchSpeaker(data) {
    console.info("[orb] speaker ←", data.name ?? "unrecognized");
    this.#speakerListeners.forEach((h) => this.#call(h, data));
  }

  dispatchEnrollment(data) {
    this.#lastEnrollmentEvent = data;
    console.info("[orb] enrollment ←", data.stage, data);
    this.#enrollmentListeners.forEach((h) => this.#call(h, data));
  }
}

export const commandBus = new CommandBus();

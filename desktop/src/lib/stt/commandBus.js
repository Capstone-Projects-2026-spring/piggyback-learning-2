class CommandBus {
  #handlers = new Map();
  #wildcards = new Set();
  #transcriptListeners = new Set();
  #wakeListeners = new Set();
  #enrollmentListeners = new Set();
  #lastEnrollmentEvent = null;

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

  onEnrollment(handler) {
    this.#enrollmentListeners.add(handler);
    // Only replay parent enrollment events — those can arrive before the
    // subscriber registers. Kid enrollment is triggered while the app is
    // already in "ready" mode so the overlay is mounted first.
    if (this.#lastEnrollmentEvent?.flow === "parent") {
      try {
        handler(this.#lastEnrollmentEvent);
      } catch (e) {
        console.error(e);
      }
    }
    return () => this.#enrollmentListeners.delete(handler);
  }

  dispatch({ intent, args, raw }) {
    const event = { intent, args, raw };
    console.info("[orb Bus] command ←", event);
    this.#handlers.get(intent)?.forEach((h) => {
      try {
        h(event);
      } catch (e) {
        console.error(e);
      }
    });
    this.#wildcards.forEach((h) => {
      try {
        h(event);
      } catch (e) {
        console.error(e);
      }
    });
  }

  dispatchTranscript(text) {
    this.#transcriptListeners.forEach((h) => {
      try {
        h(text);
      } catch (e) {
        console.error(e);
      }
    });
  }

  dispatchWake(hasEmbedding) {
    console.info("[orb Bus] wake ← embedding present:", hasEmbedding);
    this.#wakeListeners.forEach((h) => {
      try {
        h(hasEmbedding);
      } catch (e) {
        console.error(e);
      }
    });
  }

  dispatchEnrollment(data) {
    // Always cache — but onEnrollment() only replays parent flow events
    this.#lastEnrollmentEvent = data;
    console.info("[orb Bus] enrollment ←", data);
    this.#enrollmentListeners.forEach((h) => {
      try {
        h(data);
      } catch (e) {
        console.error(e);
      }
    });
  }
}

export const commandBus = new CommandBus();

export const STATUS_LABEL = {
  listening: "Listening…",
  processing: "Processing…",
  speaking: "Jarvis is speaking…",
};

export const RING_CLASS = {
  listening: "",
  processing: "ring-4 ring-pink-300 ring-offset-4 animate-pulse",
  speaking: "ring-4 ring-blue-300 ring-offset-4",
};

// Mirrors Intent enum in intent.rs — WakeOnly and Unhandled are never dispatched.
export const INTENT_RESPONSES = {
  search: "Searching for that.",
  add_kid: "Let's add a new kid profile!",
  my_answers: "Let me pull up your results.",
  add_tag: "Adding that tag now.",
  my_videos: "Here are your assigned videos.",
  assign_video: "Assigning that video now.",
  watch_video: "Loading the video for you.",
  recommendations: "Let me find something good for you!",
  download_video: "Downloading that video now.",
};

export const STARTUP_DELAY_MS = 1500;

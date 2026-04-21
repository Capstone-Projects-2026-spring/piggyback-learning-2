import { useEffect, useState, useCallback } from "react";
import { startPeppa, stopPeppa, commandBus } from "../lib/stt/index.js";
import SessionBadge from "./SessionBadge.jsx";
import peppaPng from "../assets/peppa.png";

const STATUS_LABEL = {
  listening: "Listening…",
  processing: "Processing…",
  speaking: "Peppa is speaking…",
};

const RING_CLASS = {
  listening: "",
  processing: "ring-4 ring-pink-300 ring-offset-4 animate-pulse",
  speaking: "ring-4 ring-blue-300 ring-offset-4",
};

const INTENT_RESPONSES = {
  my_tags: "Let me check your interests!",
  my_videos: "Here are your assigned videos.",
  recommendations: "Let me find something good for you!",
  my_kids: "Fetching your kids' profiles.",
  get_questions: "Here comes a quiz!",
  generate_questions: "Generating questions for you.",
  download_video: "Downloading that video now.",
  all_tags: "Here are all the available tags.",
  extract_frames: "Extracting frames from the video.",
  submit_answer: "Got your answer, checking it now!",
  my_answers: "Let me pull up your results.",
  help: "Say a command like 'show my videos' or 'quiz me'.",
  play: "Playing now!",
  stop: "Stopping.",
  search: "Searching for that.",
};

export default function PeppaOrb() {
  const [status, setStatus] = useState("listening");
  const [transcript, setTranscript] = useState("");
  const [lastCommand, setLastCommand] = useState(null);

  const speak = useCallback((text) => {
    speechSynthesis.cancel();
    const utt = new SpeechSynthesisUtterance(text);
    utt.pitch = 1.4;
    utt.rate = 1.05;
    utt.onstart = () => setStatus("speaking");
    utt.onend = () => setStatus("listening");
    speechSynthesis.speak(utt);
  }, []);

  useEffect(() => {
    let stopped = false;
    const init = async () => {
      await new Promise((res) => setTimeout(res, 1500));
      if (stopped) return;
      await startPeppa().catch((err) =>
        console.error("[PeppaOrb] startPeppa failed:", err),
      );
    };
    init();
    return () => {
      stopped = true;
      stopPeppa();
    };
  }, []);

  useEffect(() => {
    const offs = [
      commandBus.onTranscript((text) => setTranscript(text)),

      commandBus.onWake((hasEmbedding) => {
        setStatus("processing");
        if (hasEmbedding) {
          speak("I recognise you! Go ahead.");
        } else {
          speak("Hey! I'm here.");
        }
        setTimeout(() => setStatus("listening"), 1500);
      }),

      commandBus.on("*", (payload) => {
        setLastCommand(payload);
        setStatus("processing");
        const response = INTENT_RESPONSES[payload.intent];
        if (response) speak(response);
        setTimeout(() => setStatus("listening"), 1500);
      }),

      commandBus.on("chat", (p) => speak(p?.raw || "Sure!")),
    ];

    return () => offs.forEach((off) => off());
  }, [speak]);

  return (
    <div className="flex flex-col items-center justify-center min-h-screen bg-gradient-to-b from-pink-50 to-white select-none">
      {/* Session badge */}
      <div className="absolute top-6 right-6">
        <SessionBadge />
      </div>

      {/* Peppa image with status ring */}
      <div
        className={`rounded-full transition-all duration-300 ${RING_CLASS[status]}`}
      >
        <img
          src={peppaPng}
          alt="Peppa"
          className="w-48 h-48 object-contain drop-shadow-md"
          draggable={false}
        />
      </div>

      <p className="mt-6 text-sm font-medium text-pink-400 tracking-wide">
        {STATUS_LABEL[status]}
      </p>

      {/* Live transcript */}
      {transcript && (
        <div className="mt-4 max-w-xs px-4 py-2 bg-white rounded-2xl shadow-sm border border-pink-100">
          <p className="text-sm text-gray-400 text-center italic">
            "{transcript}"
          </p>
        </div>
      )}

      {/* Last matched command */}
      {lastCommand && lastCommand.intent !== "chat" && (
        <div className="mt-3 flex items-center gap-2 px-3 py-1.5 bg-pink-50 rounded-full border border-pink-200">
          <span className="text-xs text-pink-500 font-medium">
            {lastCommand.intent}
          </span>
          {lastCommand.args?.length > 0 && (
            <span className="text-xs text-pink-300">
              — {lastCommand.args.join(" ")}
            </span>
          )}
        </div>
      )}
    </div>
  );
}

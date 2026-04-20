import { useEffect, useState, useCallback } from "react";
import { startPeppa, stopPeppa, commandBus } from "../lib/stt/index.js";
import peppaPng from "../assets/peppa.png";

export default function PeppaOrb() {
  const [status, setStatus] = useState("listening");
  const [transcript, setTranscript] = useState("");
  const [lastCommand, setLastCommand] = useState(null);

  const statusLabel = {
    listening: "Listening…",
    processing: "Processing…",
    speaking: "Peppa is speaking…",
  };

  const ringClass = {
    listening: "",
    processing: "ring-4 ring-pink-300 ring-offset-4 animate-pulse",
    speaking: "ring-4 ring-blue-300 ring-offset-4",
  };

  const speak = useCallback((text) => {
    speechSynthesis.cancel();
    const utt = new SpeechSynthesisUtterance(text);
    utt.pitch = 1.4;
    utt.rate = 1.05;
    utt.onstart = () => setStatus("speaking");
    utt.onend = () => setStatus("listening");
    speechSynthesis.speak(utt);
  }, []);

  // Single effect for mic init — delay gives PipeWire time to settle
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

  // Command bus wiring
  useEffect(() => {
    const offTranscript = commandBus.onTranscript((text) => {
      setTranscript(text);
    });

    const offAll = commandBus.on("*", (payload) => {
      console.log("[PeppaOrb] intent received:", payload);
      setLastCommand(payload);
      setStatus("processing");
      setTimeout(() => setStatus("listening"), 1200);
    });

    const offHelp = commandBus.on("help", () =>
      speak("Say hey Peppa followed by open, play, search, or chat."),
    );
    const offWake = commandBus.on("wake_only", () => speak("Yes? I'm here!"));
    const offChat = commandBus.on("chat", (p) => speak(p?.raw || "Sure!"));

    return () => {
      offTranscript();
      offAll();
      offHelp();
      offWake();
      offChat();
    };
  }, [speak]);

  return (
    <div className="flex flex-col items-center justify-center min-h-screen bg-gradient-to-b from-pink-50 to-white select-none">
      <div
        className={`rounded-full transition-all duration-300 ${ringClass[status]}`}
      >
        <img
          src={peppaPng}
          alt="Peppa"
          className="w-48 h-48 object-contain drop-shadow-md"
          draggable={false}
        />
      </div>

      <p className="mt-6 text-sm font-medium text-pink-400 tracking-wide">
        {statusLabel[status]}
      </p>

      {transcript && (
        <div className="mt-4 max-w-xs px-4 py-2 bg-white rounded-2xl shadow-sm border border-pink-100">
          <p className="text-sm text-gray-500 text-center italic">
            "{transcript}"
          </p>
        </div>
      )}

      {lastCommand && (
        <div className="mt-3 flex items-center gap-2 px-3 py-1.5 bg-pink-50 rounded-full border border-pink-200">
          <span className="text-xs text-pink-500 font-medium">
            {lastCommand.intent}
          </span>
          {lastCommand.raw && (
            <span className="text-xs text-pink-300">— {lastCommand.raw}</span>
          )}
        </div>
      )}
    </div>
  );
}

import { useEffect, useState, useRef } from "react";
import { startOrb, stopOrb, commandBus } from "@/lib";
import { speak } from "@/utils";
import SessionBadge from "./SessionBadge.jsx";
import OrbVisual from "./OrbVisual.jsx";
import OrbTranscript from "./OrbTranscript.jsx";
import { INTENT_RESPONSES, STARTUP_DELAY_MS } from "@/utils";

export default function Orb() {
  const [status, setStatus] = useState("listening");
  const [transcript, setTranscript] = useState("");
  const [lastCommand, setLastCommand] = useState(null);

  useEffect(() => {
    let cancelled = false;
    const init = async () => {
      await new Promise((res) => setTimeout(res, STARTUP_DELAY_MS));
      if (cancelled) return;
      await startOrb().catch((err) =>
        console.error("[orb] startOrb failed:", err),
      );
    };
    init();
    return () => {
      cancelled = true;
      stopOrb();
    };
  }, []);

  useEffect(() => {
    const offs = [
      commandBus.onTranscript((text) => setTranscript(text)),

      commandBus.onWake((hasEmbedding) => {
        setStatus("processing");
        setTimeout(() => setStatus("listening"), 1500);
      }),

      commandBus.on("*", (payload) => {
        if (payload.intent === "chat") return;
        setLastCommand(payload);
        setStatus("processing");
        const response = INTENT_RESPONSES[payload.intent];
        if (response) speak(response);
        setTimeout(() => setStatus("listening"), 1500);
      }),

      commandBus.on("chat", (p) => speak(p?.raw || "Sure!")),
    ];

    return () => offs.forEach((off) => off());
  }, []);

  return (
    <div className="flex flex-col items-center justify-center min-h-screen bg-gradient-to-b from-pink-50 to-white select-none">
      <div className="absolute top-6 right-6">
        <SessionBadge />
      </div>
      <OrbVisual status={status} />
      <OrbTranscript transcript={transcript} lastCommand={lastCommand} />
    </div>
  );
}

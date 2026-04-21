import { useEffect, useState } from "react";
import { commandBus } from "../lib/stt/commandBus.js";

export default function SessionBadge() {
  const [identified, setIdentified] = useState(false);
  const [flashWake, setFlashWake] = useState(false);

  useEffect(() => {
    const off = commandBus.onWake((hasEmbedding) => {
      if (hasEmbedding) setIdentified(true);
      // Flash the ring briefly to show wake was detected
      setFlashWake(true);
      setTimeout(() => setFlashWake(false), 1000);
    });
    return off;
  }, []);

  if (!identified && !flashWake) return null;

  return (
    <div
      className={`
      flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium
      transition-all duration-300
      ${
        identified
          ? "bg-green-50 border border-green-200 text-green-600"
          : "bg-pink-50 border border-pink-200 text-pink-500"
      }
    `}
    >
      <span
        className={`w-1.5 h-1.5 rounded-full ${identified ? "bg-green-400" : "bg-pink-400 animate-pulse"}`}
      />
      {identified ? "voice recognised" : "identifying…"}
    </div>
  );
}

import { useEffect, useState } from "react";
import { commandBus } from "@/lib";
import { speak } from "@/utils";

export default function SessionBadge() {
  const [speaker, setSpeaker] = useState(null);
  const [flashWake, setFlashWake] = useState(false);

  useEffect(() => {
    let flashTimer = null;

    const offWake = commandBus.onWake(() => {
      setFlashWake(true);
      flashTimer = setTimeout(() => setFlashWake(false), 1000);
    });

    const offSpeaker = commandBus.onSpeaker((data) => {
      if (data?.user_id) {
        setSpeaker(data);
        speak(`Welcome back, ${data.name}!`);
      }
    });

    return () => {
      offWake();
      offSpeaker();
      clearTimeout(flashTimer);
    };
  }, []);

  if (!speaker && !flashWake) return null;

  const identified = !!speaker;

  return (
    <div
      className={`flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium transition-all duration-300 ${
        identified
          ? "bg-green-50 border border-green-200 text-green-600"
          : "bg-pink-50 border border-pink-200 text-pink-500"
      }`}
    >
      <span
        className={`w-1.5 h-1.5 rounded-full ${identified ? "bg-green-400" : "bg-pink-400 animate-pulse"}`}
      />
      {identified ? speaker.name : "identifying…"}
    </div>
  );
}

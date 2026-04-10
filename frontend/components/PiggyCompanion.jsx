"use client";

import { useEffect, useState } from "react";
import Image from "next/image";

import piggy from "../animations/piggy.png";
import piggyTalk from "../animations/piggyc_talk.png";

export default function PiggyCompanion({ mode = "watch", text = "" }) {
  const [currentPiggy, setCurrentPiggy] = useState(piggy);

  useEffect(() => {
    if (mode !== "talk") {
      setCurrentPiggy(piggy);
      return;
    }

    const interval = setInterval(() => {
      setCurrentPiggy((prev) => (prev === piggy ? piggyTalk : piggy));
    }, 400);

    return () => clearInterval(interval);
  }, [mode]);

  if (mode === "hidden") return null;
  

  return (
    <div className="flex flex-col items-center gap-2">
      {text && (
        <div className="relative bg-white px-5 py-3 rounded-2xl shadow-lg text-sm text-gray-800 font-semibold max-w-[220px] text-center">
  {text}

  
  <div className="absolute bottom-[-6px] left-1/2 -translate-x-1/2 w-3 h-3 bg-white rotate-45 border-r border-b border-gray-100"></div>
</div>
      )}

      <div className="animate-[float_3s_ease-in-out_infinite]">
        <Image
          src={currentPiggy}
          alt="Piggy companion"
          width={160}
          height={160}
          className="object-contain"
        />
      </div>
    </div>
  );
}
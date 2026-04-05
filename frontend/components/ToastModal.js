"use client";

import { useEffect } from "react";

export default function ToastModal({ data, onClose, getStyle }) {
  const style = getStyle(data);

  useEffect(() => {
    if (!data) return;

    const audio = new Audio("/alert.mp3");
    audio.volume = 1.0;

    audio.play().catch(() => {
      // ignore autoplay block errors
    });

    return () => {
      audio.pause();
      audio.currentTime = 0;
    };
  }, [data]);

  return (
    <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
      <div
        className={`w-[90%] max-w-md p-6 rounded-2xl shadow-xl border-2 ${style.bg} ${style.border}`}
      >
        <div className="flex justify-between items-center mb-4">
          <h2 className={`text-xl font-bold ${style.text}`}>
            {style.emoji} {style.title}
          </h2>
          <button
            onClick={onClose}
            className="text-gray-500 hover:text-black text-lg"
          >
            ✖
          </button>
        </div>

        <p className={`text-lg ${style.text}`}>
          {data?.msg || "You have a new notification"}
        </p>
      </div>
    </div>
  );
}

import { useEffect, useState } from "react";

const CHARACTERS = ["🦁", "🐸", "🦊", "🐧", "🦄"];
const MESSAGES = [
  "Hey! Come back, the fun part is right here! 👀",
  "Oops! Looks like you wandered off — the video misses you!",
  "Yoo-hoo! Your eyes went on an adventure! Come back! 🗺️",
  "The video is waiting for you! Let's keep watching! 🍿",
];

export default function LookAtScreenModal({ visible }) {
  const [character] = useState(
    () => CHARACTERS[Math.floor(Math.random() * CHARACTERS.length)],
  );
  const [message] = useState(
    () => MESSAGES[Math.floor(Math.random() * MESSAGES.length)],
  );
  const [bounce, setBounce] = useState(false);

  // Bounce the character every 1.5s to stay lively
  useEffect(() => {
    if (!visible) return;
    const interval = setInterval(() => setBounce((b) => !b), 750);
    return () => clearInterval(interval);
  }, [visible]);

  if (!visible) return null;

  return (
    <div className="fixed inset-0 z-60 flex items-center justify-center">
      {/* Soft blurred backdrop */}
      <div className="absolute inset-0 bg-black bg-opacity-40 backdrop-blur-sm" />

      <div
        className="relative bg-white rounded-3xl shadow-2xl px-10 py-10 flex flex-col items-center gap-5 max-w-sm w-full mx-4"
        style={{ animation: "popIn 0.4s cubic-bezier(0.34,1.56,0.64,1) both" }}
      >
        {/* Animated character */}
        <div
          className="text-8xl select-none"
          style={{
            transition: "transform 0.3s ease",
            transform: bounce
              ? "translateY(-14px) rotate(-8deg)"
              : "translateY(0px) rotate(8deg)",
            filter: "drop-shadow(0 8px 12px rgba(0,0,0,0.15))",
          }}
        >
          {character}
        </div>

        {/* Stars decoration */}
        <div
          className="flex gap-2 text-2xl"
          style={{ animation: "sparkle 1s ease-in-out infinite alternate" }}
        >
          {"⭐✨⭐".split("").map((s, i) => (
            <span key={i} style={{ animationDelay: `${i * 0.2}s` }}>
              {s}
            </span>
          ))}
        </div>

        <h2 className="text-2xl font-extrabold text-purple-700 text-center leading-snug">
          Where did you go?
        </h2>

        <p className="text-gray-600 text-center text-base leading-relaxed">
          {message}
        </p>

        {/* Pulsing "eyes here" indicator */}
        <div className="flex items-center gap-2 bg-purple-100 rounded-full px-5 py-2">
          <span
            className="text-xl"
            style={{ animation: "blink 1s step-end infinite" }}
          >
            👀
          </span>
          <span className="text-purple-700 font-semibold text-sm">
            Look at the screen!
          </span>
          <span
            className="text-xl"
            style={{ animation: "blink 1s step-end infinite 0.5s" }}
          >
            👀
          </span>
        </div>

        <p className="text-xs text-gray-400">
          The video will start again when you&apos;re back 🎬
        </p>
      </div>

      <style>{`
        @keyframes popIn {
          from { opacity: 0; transform: scale(0.6); }
          to   { opacity: 1; transform: scale(1); }
        }
        @keyframes blink {
          0%, 100% { opacity: 1; }
          50%       { opacity: 0.2; }
        }
        @keyframes sparkle {
          from { transform: scale(1) rotate(-5deg); }
          to   { transform: scale(1.15) rotate(5deg); }
        }
      `}</style>
    </div>
  );
}

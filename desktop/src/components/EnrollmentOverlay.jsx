import { useEffect, useState, useRef } from "react";
import { commandBus } from "../lib/stt/commandBus.js";
import peppaPng from "../assets/peppa.png";

export default function EnrollmentOverlay({ onDone }) {
  const [stage, setStage] = useState("greet");
  const [message, setMessage] = useState("");
  const [prompts, setPrompts] = useState([]);
  const [activeIdx, setActiveIdx] = useState(0);
  const [parentName, setParentName] = useState("");
  const spokenRef = useRef(false);

  const speak = (text) => {
    if (!text || spokenRef.current) return;
    speechSynthesis.cancel();
    const utt = new SpeechSynthesisUtterance(text);
    utt.pitch = 1.4;
    utt.rate = 1.05;
    spokenRef.current = true;
    utt.onend = () => {
      spokenRef.current = false;
    };
    speechSynthesis.speak(utt);
  };

  useEffect(() => {
    const off = commandBus.onEnrollment((data) => {
      setStage(data.stage);
      setMessage(data.message);
      if (data.prompts?.length) setPrompts(data.prompts);
      setActiveIdx(data.prompt_index ?? 0);

      if (data.stage === "greet") {
        speak(data.message);
      } else if (data.stage === "name_confirmed") {
        const match = data.message.match(/you,\s+([^!]+)!/);
        if (match) setParentName(match[1].trim());
        speak(data.message);
      } else if (data.stage === "prompt") {
        const phrase = data.prompts?.[data.prompt_index];
        speak(phrase ? `Read this: ${phrase}` : data.message);
      } else if (data.stage === "done") {
        speak("You're all set! I'll recognise your voice from now on.");
        setTimeout(() => onDone?.(), 3000);
      } else {
        speak(data.message);
      }
    });
    return off;
  }, [onDone]);

  const completedCount =
    stage === "prompt" ? activeIdx : stage === "done" ? prompts.length : 0;

  // Startup placeholder before first event arrives
  if (!message) {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen bg-gradient-to-b from-pink-50 to-white select-none">
        <img
          src={peppaPng}
          alt="Peppa"
          className="w-36 h-36 object-contain"
          draggable={false}
        />
        <p className="mt-6 text-sm text-pink-300 animate-pulse">Starting up…</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center min-h-screen bg-gradient-to-b from-pink-50 to-white select-none px-6 py-12">
      {/* Peppa orb */}
      <div
        className={`rounded-full transition-all duration-500 ${
          stage === "prompt"
            ? "ring-4 ring-blue-200 ring-offset-4 animate-pulse"
            : stage === "done"
              ? "ring-4 ring-green-300 ring-offset-4"
              : stage === "name_confirmed"
                ? "ring-4 ring-pink-200 ring-offset-4"
                : stage === "greet"
                  ? "ring-4 ring-pink-100 ring-offset-4"
                  : ""
        }`}
      >
        <img
          src={peppaPng}
          alt="Peppa"
          className="w-36 h-36 object-contain"
          draggable={false}
        />
      </div>

      {/* Status pill */}
      <div className="mt-5 px-4 py-1.5 rounded-full bg-pink-100 border border-pink-200">
        <p className="text-xs font-medium text-pink-500 tracking-wide">
          {stage === "greet" && "getting started"}
          {stage === "name_confirmed" && `hey, ${parentName || "there"}!`}
          {stage === "prompt" &&
            `voice setup — ${completedCount} of ${prompts.length} done`}
          {stage === "done" && "all set!"}
          {stage === "error" && "something went wrong"}
        </p>
      </div>

      {/* Main message bubble */}
      <div className="mt-5 max-w-sm w-full px-5 py-4 bg-white rounded-3xl border border-pink-100 text-center">
        <p className="text-sm text-gray-500 leading-relaxed">
          {stage === "greet"
            ? message
            : stage === "name_confirmed"
              ? message
              : stage === "prompt"
                ? "Read each sentence out loud. Peppa will move on automatically when she hears you."
                : message}
        </p>
      </div>

      {/* Listening pulse for name stage */}
      {stage === "greet" && (
        <div className="mt-5 flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-pink-400 animate-pulse" />
          <p className="text-xs text-gray-400">
            Peppa is listening for your name…
          </p>
        </div>
      )}

      {/* Phrase list — shown during name_confirmed and prompt stages */}
      {prompts.length > 0 &&
        (stage === "name_confirmed" || stage === "prompt") && (
          <div className="mt-6 w-full max-w-sm flex flex-col gap-3">
            <p className="text-xs text-gray-400 text-center tracking-widest uppercase mb-1">
              phrases to read
            </p>
            {prompts.map((phrase, i) => {
              const done = i < completedCount;
              const active = i === completedCount && stage === "prompt";
              const pending = !done && !active;

              return (
                <div
                  key={i}
                  className={`flex items-start gap-3 px-4 py-3 rounded-2xl border transition-all duration-300 ${
                    done
                      ? "bg-green-50 border-green-200 opacity-70"
                      : active
                        ? "bg-blue-50 border-blue-300 shadow-sm"
                        : "bg-white border-gray-100 opacity-40"
                  }`}
                >
                  <div
                    className={`mt-0.5 w-5 h-5 rounded-full flex items-center justify-center flex-shrink-0 ${
                      done
                        ? "bg-green-400"
                        : active
                          ? "bg-blue-400 animate-pulse"
                          : "bg-gray-200"
                    }`}
                  >
                    {done ? (
                      <svg
                        className="w-3 h-3 text-white"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={3}
                          d="M5 13l4 4L19 7"
                        />
                      </svg>
                    ) : (
                      <span className="text-xs font-medium text-white">
                        {i + 1}
                      </span>
                    )}
                  </div>
                  <p
                    className={`text-sm leading-snug ${
                      active
                        ? "text-blue-700 font-medium"
                        : done
                          ? "text-green-700"
                          : "text-gray-400"
                    }`}
                  >
                    "{phrase}"
                  </p>
                </div>
              );
            })}
          </div>
        )}

      {/* Done state */}
      {stage === "done" && (
        <div className="mt-8 flex flex-col items-center gap-3">
          <div className="w-14 h-14 rounded-full bg-green-100 flex items-center justify-center">
            <svg
              className="w-7 h-7 text-green-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M5 13l4 4L19 7"
              />
            </svg>
          </div>
          <p className="text-sm text-green-600 font-medium">
            Voice profile saved!
          </p>
          <p className="text-xs text-gray-400">
            Say "hey Peppa" any time to wake me up.
          </p>
        </div>
      )}

      {/* Error state */}
      {stage === "error" && (
        <div className="mt-8 flex flex-col items-center gap-3">
          <div className="w-14 h-14 rounded-full bg-red-100 flex items-center justify-center">
            <svg
              className="w-7 h-7 text-red-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </div>
          <p className="text-sm text-red-500">{message}</p>
        </div>
      )}
    </div>
  );
}

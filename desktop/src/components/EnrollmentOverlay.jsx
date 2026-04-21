import { useEffect, useState, useRef } from "react";
import { commandBus } from "../lib/stt/commandBus.js";
import peppaPng from "../assets/peppa.png";

// Stage strings coming from Rust per flow:
// parent: greet | name_confirmed | prompt | done | error
// kid:    kid_greet | kid_name_confirmed | kid_prompt | kid_done | error
function normaliseStage(stage) {
  return stage.replace(/^kid_/, "");
}

const THEME = {
  parent: {
    bg: "from-pink-50 to-white",
    pill: "bg-pink-100 border-pink-200 text-pink-500",
    orb: {
      greet: "ring-pink-100",
      name_confirmed: "ring-pink-200",
      prompt: "ring-blue-200",
      done: "ring-green-300",
    },
    listening: "text-gray-400",
    dot: "bg-pink-400",
  },
  kid: {
    bg: "from-violet-50 to-white",
    pill: "bg-violet-100 border-violet-200 text-violet-500",
    orb: {
      greet: "ring-violet-100",
      name_confirmed: "ring-violet-200",
      prompt: "ring-blue-200",
      done: "ring-green-300",
    },
    listening: "text-gray-400",
    dot: "bg-violet-400",
  },
};

export default function EnrollmentOverlay({ flow = "parent", onDone }) {
  const [stage, setStage] = useState("greet");
  const [message, setMessage] = useState("");
  const [prompts, setPrompts] = useState([]);
  const [activeIdx, setActiveIdx] = useState(0);
  const [userName, setUserName] = useState("");
  const spokenRef = useRef(false);
  const theme = THEME[flow] ?? THEME.parent;

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
      // Only handle events for our flow
      if (data.flow !== flow) return;

      const normalised = normaliseStage(data.stage);
      setStage(normalised);
      setMessage(data.message);
      if (data.prompts?.length) setPrompts(data.prompts);
      setActiveIdx(data.prompt_index ?? 0);

      if (normalised === "greet") {
        speak(data.message);
      } else if (normalised === "name_confirmed") {
        const match = data.message.match(/you,\s+([^!]+)!/);
        if (match) setUserName(match[1].trim());
        speak(data.message);
      } else if (normalised === "prompt") {
        const phrase = data.prompts?.[data.prompt_index];
        speak(phrase ? `Read this: ${phrase}` : data.message);
      } else if (normalised === "done") {
        speak(data.message);
        setTimeout(() => onDone?.(), 3000);
      } else {
        speak(data.message);
      }
    });
    return off;
  }, [flow, onDone]);

  const completedCount =
    stage === "prompt" ? activeIdx : stage === "done" ? prompts.length : 0;

  const pillLabel = () => {
    if (stage === "greet")
      return flow === "kid" ? "new kid setup" : "getting started";
    if (stage === "name_confirmed") return `hey, ${userName || "there"}!`;
    if (stage === "prompt")
      return `voice setup — ${completedCount} of ${prompts.length} done`;
    if (stage === "done") return "all set!";
    if (stage === "error") return "something went wrong";
    return "";
  };

  if (!message) {
    return (
      <div
        className={`flex flex-col items-center justify-center min-h-screen bg-gradient-to-b ${theme.bg} select-none`}
      >
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

  const orbRing = theme.orb[stage] ?? "";

  return (
    <div
      className={`flex flex-col items-center min-h-screen bg-gradient-to-b ${theme.bg} select-none px-6 py-12`}
    >
      {/* Peppa orb */}
      <div
        className={`rounded-full transition-all duration-500 ${
          stage === "greet" ||
          stage === "name_confirmed" ||
          stage === "prompt" ||
          stage === "done"
            ? `ring-4 ring-offset-4 ${orbRing} ${stage === "prompt" ? "animate-pulse" : ""}`
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
      <div className={`mt-5 px-4 py-1.5 rounded-full border ${theme.pill}`}>
        <p className="text-xs font-medium tracking-wide">{pillLabel()}</p>
      </div>

      {/* Main message bubble */}
      <div className="mt-5 max-w-sm w-full px-5 py-4 bg-white rounded-3xl border border-gray-100 text-center">
        <p className="text-sm text-gray-500 leading-relaxed">
          {stage === "prompt"
            ? "Read each sentence out loud. Peppa will move on automatically when she hears you."
            : message}
        </p>
      </div>

      {/* Listening pulse — greet stage */}
      {stage === "greet" && (
        <div className="mt-5 flex items-center gap-2">
          <span className={`w-2 h-2 rounded-full animate-pulse ${theme.dot}`} />
          <p className={`text-xs ${theme.listening}`}>
            {flow === "kid"
              ? "Peppa is listening for the kid's name…"
              : "Peppa is listening for your name…"}
          </p>
        </div>
      )}

      {/* Phrase list */}
      {prompts.length > 0 &&
        (stage === "name_confirmed" || stage === "prompt") && (
          <div className="mt-6 w-full max-w-sm flex flex-col gap-3">
            <p className="text-xs text-gray-400 text-center tracking-widest uppercase mb-1">
              phrases to read
            </p>
            {prompts.map((phrase, i) => {
              const done = i < completedCount;
              const active = i === completedCount && stage === "prompt";

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
            {flow === "kid"
              ? `Say "hey Peppa" to start learning!`
              : `Say "hey Peppa" any time to wake me up.`}
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

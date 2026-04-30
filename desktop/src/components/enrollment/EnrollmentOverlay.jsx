import { useEffect, useState, useRef } from "react";
import orbPng from "@/assets/orb.png";
import OrbDisplay from "./OrbDisplay.jsx";
import PhraseList from "./PhraseList.jsx";
import StatusFooter from "./StatusFooter.jsx";
import { normaliseStage, speak, THEME } from "@/utils";

export default function EnrollmentOverlay({
  flow = "parent",
  currentEvent,
  onDone,
}) {
  const [stage, setStage] = useState("greet");
  const [message, setMessage] = useState("");
  const [prompts, setPrompts] = useState([]);
  const [activeIdx, setActiveIdx] = useState(0);
  const [userName, setUserName] = useState("");

  const onDoneRef = useRef(onDone);
  useEffect(() => {
    onDoneRef.current = onDone;
  }, [onDone]);

  const theme = THEME[flow] ?? THEME.parent;

  useEffect(() => {
    if (!currentEvent) return;
    if (currentEvent.flow !== flow) return;

    const normalised = normaliseStage(currentEvent.stage);
    setStage(normalised);
    setMessage(currentEvent.message);
    if (currentEvent.prompts?.length) setPrompts(currentEvent.prompts);
    const idx = currentEvent.prompt_index ?? 0;
    setActiveIdx(idx);

    if (normalised === "name_confirmed") {
      const match = currentEvent.message.match(/you,\s+([^!]+)!/);
      if (match) setUserName(match[1].trim());
      speak(currentEvent.message);
    } else if (normalised === "prompt") {
      const phrase = currentEvent.prompts?.[idx];
      speak(phrase ? `Read this: ${phrase}` : currentEvent.message);
    } else if (normalised === "done") {
      speak(currentEvent.message);
      setTimeout(() => onDoneRef.current?.(), 3000);
    } else if (normalised === "greet") {
      speak(currentEvent.message);
    }
  }, [currentEvent, flow]);

  const pillLabel = () => {
    if (stage === "greet")
      return flow === "kid" ? "new kid setup" : "getting started";
    if (stage === "name_confirmed") return `hey, ${userName || "there"}!`;
    if (stage === "prompt")
      return `voice setup - ${activeIdx} of ${prompts.length} done`;
    if (stage === "done") return "all set!";
    if (stage === "error") return "something went wrong";
    return "";
  };

  if (!message) {
    return (
      <div
        className={`flex flex-col items-center justify-center min-h-screen bg-linear-to-b ${theme.bg} select-none`}
      >
        <img
          src={orbPng}
          alt="orb"
          className="w-36 h-36 object-contain"
          draggable={false}
        />
        <div className="mt-6 flex flex-col items-center gap-3">
          <div className="flex items-center gap-2">
            <span
              className={`w-2 h-2 rounded-full animate-pulse ${theme.dot}`}
            />
            <p className="text-xs text-gray-400">
              {flow === "kid"
                ? "Jarvis is listening for the kid's name…"
                : "Jarvis is listening for your name…"}
            </p>
          </div>
          <div className={`px-4 py-1.5 rounded-full border ${theme.pill}`}>
            <p className="text-xs font-medium tracking-wide">
              {flow === "kid" ? "new kid setup" : "getting started"}
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`flex flex-col items-center min-h-screen bg-linear-to-b ${theme.bg} select-none px-6 py-12`}
    >
      <OrbDisplay stage={stage} theme={theme} />

      <div className={`mt-5 px-4 py-1.5 rounded-full border ${theme.pill}`}>
        <p className="text-xs font-medium tracking-wide">{pillLabel()}</p>
      </div>

      <div className="mt-5 max-w-sm w-full px-5 py-4 bg-white rounded-3xl border border-gray-100 text-center">
        <p className="text-sm text-gray-500 leading-relaxed">
          {stage === "prompt"
            ? "Read each sentence out loud. Jarvis will move on automatically when he hears you."
            : message}
        </p>
      </div>

      {stage === "greet" && (
        <div className="mt-5 flex items-center gap-2">
          <span className={`w-2 h-2 rounded-full animate-pulse ${theme.dot}`} />
          <p className="text-xs text-gray-400">
            {flow === "kid"
              ? "Jarvis is listening for the kid's name…"
              : "Jarvis is listening for your name…"}
          </p>
        </div>
      )}

      <PhraseList prompts={prompts} completedCount={activeIdx} stage={stage} />
      <StatusFooter stage={stage} flow={flow} message={message} />
    </div>
  );
}

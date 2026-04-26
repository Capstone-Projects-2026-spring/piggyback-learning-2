import orbPng from "@/assets/orb.png";

export default function OrbDisplay({ stage, theme }) {
  const ringClass = theme.orb[stage] ?? "";
  return (
    <div
      className={`rounded-full transition-all duration-500 ${
        ringClass ? `ring-4 ring-offset-4 ${ringClass}` : ""
      } ${stage === "prompt" ? "animate-pulse" : ""}`}
    >
      <img
        src={orbPng}
        alt="orb"
        className="w-36 h-36 object-contain"
        draggable={false}
      />
    </div>
  );
}

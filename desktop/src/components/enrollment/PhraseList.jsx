export default function PhraseList({ prompts, completedCount, stage }) {
  if (!prompts.length || (stage !== "name_confirmed" && stage !== "prompt")) {
    return null;
  }

  return (
    <div className="mt-6 w-full max-w-sm flex flex-col gap-3">
      <p className="text-xs text-gray-400 text-center tracking-widest uppercase mb-1">
        phrases to read
      </p>
      {prompts.map((phrase, i) => {
        const done = i < completedCount;
        const active = i === completedCount;
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
                <span className="text-xs font-medium text-white">{i + 1}</span>
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
  );
}

export default function StatusFooter({ stage, flow, message }) {
  if (stage === "done") {
    return (
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
            ? `Say "hey Jarvis" to start learning!`
            : `Say "hey Jarvis" any time to wake me up.`}
        </p>
      </div>
    );
  }

  if (stage === "error") {
    return (
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
    );
  }

  return null;
}

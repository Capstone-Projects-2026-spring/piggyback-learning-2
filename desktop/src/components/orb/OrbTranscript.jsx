export default function OrbTranscript({ transcript, lastCommand }) {
  return (
    <>
      {transcript && (
        <div className="mt-4 max-w-xs px-4 py-2 bg-white rounded-2xl shadow-sm border border-pink-100">
          <p className="text-sm text-gray-400 text-center italic">
            "{transcript}"
          </p>
        </div>
      )}

      {lastCommand && lastCommand.intent !== "chat" && (
        <div className="mt-3 flex items-center gap-2 px-3 py-1.5 bg-pink-50 rounded-full border border-pink-200">
          <span className="text-xs text-pink-500 font-medium">
            {lastCommand.intent}
          </span>
          {lastCommand.args?.length > 0 && (
            <span className="text-xs text-pink-300">
              — {lastCommand.args.join(" ")}
            </span>
          )}
        </div>
      )}
    </>
  );
}

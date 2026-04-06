export default function RecordingStatusBadge({
  recordingState,
  statusMessage,
}) {
  if (recordingState === "idle") return null;

  return (
    <div
      className={`mb-4 px-4 py-2 rounded-full text-sm font-semibold shadow-md transition-all ${
        recordingState === "waiting"
          ? "bg-yellow-300 text-yellow-900"
          : recordingState === "recording"
            ? "bg-red-500 text-white animate-pulse"
            : "bg-blue-400 text-white"
      }`}
    >
      {statusMessage}
    </div>
  );
}

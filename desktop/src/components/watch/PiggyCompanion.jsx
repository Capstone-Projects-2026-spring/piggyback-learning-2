export default function PiggyCompanion({ mode, text }) {
  if (mode === "hidden" || mode === "loading") return null;
  return (
    <div className="absolute bottom-6 right-6 flex flex-col items-end gap-2 pointer-events-none z-10">
      {mode === "talk" && text && (
        <div className="max-w-xs bg-white/90 backdrop-blur-sm rounded-2xl rounded-br-sm px-3 py-2 shadow-lg">
          <p className="text-xs text-gray-700 font-medium leading-snug">
            {text}
          </p>
        </div>
      )}
      <div className="w-14 h-14 rounded-full bg-pink-100 border-2 border-pink-200 flex items-center justify-center text-2xl shadow-lg">
        🐷
      </div>
    </div>
  );
}

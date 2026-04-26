export default function ProgressBar({ accuracy }) {
  const color =
    accuracy >= 80
      ? "bg-green-400"
      : accuracy >= 50
        ? "bg-amber-400"
        : "bg-red-400";
  return (
    <div className="mt-3 h-2 w-full rounded-full bg-gray-100">
      <div
        className={`h-2 rounded-full transition-all duration-500 ${color}`}
        style={{ width: `${accuracy}%` }}
      />
    </div>
  );
}

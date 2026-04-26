export default function ScoreBadge({ accuracy }) {
  const color =
    accuracy >= 80
      ? "text-green-600"
      : accuracy >= 50
        ? "text-amber-500"
        : "text-red-500";
  return <span className={`text-4xl font-bold ${color}`}>{accuracy}%</span>;
}

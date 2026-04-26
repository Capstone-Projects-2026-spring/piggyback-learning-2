export default function Spinner({
  className = "w-8 h-8 border-pink-200 border-t-pink-400",
}) {
  return <div className={`rounded-full border-2 animate-spin ${className}`} />;
}

export default function PulseDot({
  className = "bg-pink-400",
  size = "w-2 h-2",
}) {
  return <span className={`rounded-full animate-pulse ${size} ${className}`} />;
}

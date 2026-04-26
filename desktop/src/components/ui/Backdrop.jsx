export default function Backdrop({
  onClick,
  blur = true,
  dark = "bg-black/50",
}) {
  return (
    <div
      className={`absolute inset-0 ${dark} ${blur ? "backdrop-blur-sm" : ""}`}
      onClick={onClick}
    />
  );
}

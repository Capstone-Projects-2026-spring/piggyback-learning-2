import { STATUS_MAP } from "@/utils";

export default function VideoStatusBadge({ status }) {
  const s = STATUS_MAP[status];
  if (!s) return null;
  return (
    <span
      className={`self-start text-xs px-2 py-0.5 rounded-full border ${s.bg}`}
    >
      {s.label}
    </span>
  );
}

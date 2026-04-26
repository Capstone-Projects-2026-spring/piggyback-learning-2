import { formatTime } from "@/utils";

export default function SegmentTabs({ segments, activeIndex, onSelect }) {
  if (segments.length <= 1) return null;

  return (
    <div className="flex gap-2 px-5 py-3 overflow-x-auto shrink-0 border-b border-gray-50">
      {segments.map((seg, i) => (
        <button
          key={seg.segment.id}
          onClick={() => onSelect(i)}
          className={`shrink-0 text-xs px-3 py-1.5 rounded-full border transition-colors ${
            i === activeIndex
              ? "bg-pink-400 border-pink-400 text-white"
              : "bg-white border-gray-200 text-gray-500 hover:border-pink-200"
          }`}
        >
          {formatTime(seg.segment.start_seconds)}–
          {formatTime(seg.segment.end_seconds)}
        </button>
      ))}
    </div>
  );
}

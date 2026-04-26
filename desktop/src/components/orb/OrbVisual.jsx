import orbPng from "@/assets/orb.png";
import { STATUS_LABEL, RING_CLASS } from "@/utils";

export default function OrbVisual({ status }) {
  return (
    <>
      <div
        className={`rounded-full transition-all duration-300 ${RING_CLASS[status] ?? ""}`}
      >
        <img
          src={orbPng}
          alt="orb"
          className="w-48 h-48 object-contain drop-shadow-md"
          draggable={false}
        />
      </div>
      <p className="mt-6 text-sm font-medium text-pink-400 tracking-wide">
        {STATUS_LABEL[status]}
      </p>
    </>
  );
}

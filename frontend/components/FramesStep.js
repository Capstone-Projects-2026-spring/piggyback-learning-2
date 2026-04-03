import { useState } from "react";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function FramesStep({ videoId, setStep }) {
  const [loading, setLoading] = useState(false);

  async function handleExtract() {
    setLoading(true);

    const res = await fetch(`${BASE_URL}/api/frames/extract/${videoId}`);
    const data = await res.json();

    setLoading(false);

    if (data.success) setStep(3);
  }

  return (
    <button
      onClick={handleExtract}
      className="w-full py-3 bg-purple-500 text-white rounded-xl"
    >
      {loading ? "Extracting..." : "Extract Frames 🎞️"}
    </button>
  );
}

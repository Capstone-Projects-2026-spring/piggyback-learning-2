import { useState } from "react";
import { usePiggy } from "@/context/PiggyContext";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function DownloadStep({ videoId, setStep }) {
  const [loading, setLoading] = useState(false);
   const { setPiggyText } = usePiggy();

  async function handleDownload() {
     setPiggyText("Downloading your video... ⬇️"); 
    setLoading(true);

    const res = await fetch(`${BASE_URL}/api/videos/download/${videoId}`);
    const data = await res.json();

    setLoading(false);

    if (data.success || data.msg?.includes("already")) {
      setStep(2);
    }
  }

  return (
    <button
      onClick={handleDownload}
      className="w-full py-3 bg-blue-500 text-white rounded-xl"
    >
      {loading ? "Downloading..." : "Download Video 📥"}
    </button>
  );
}

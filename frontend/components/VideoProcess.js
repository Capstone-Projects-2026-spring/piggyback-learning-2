import DownloadStep from "./DownloadStep";
import FramesStep from "./FramesStep";
import TagsStep from "./TagsStep";
import QuestionsStep from "./QuestionsStep";
import { useState } from "react";

export default function VideoProcess({ videoId, kidId }) {
  const [step, setStep] = useState(1);

  return (
    <div className="min-h-screen bg-linear-to-br from-blue-50 via-purple-50 to-pink-50 p-6">
      <div className="max-w-3xl mx-auto bg-white rounded-3xl shadow-xl p-6">
        <h1 className="text-2xl font-bold mb-6">🎬 Process Video</h1>

        <div className="flex justify-between mb-6 text-sm font-semibold">
          {["Download", "Frames", "Tags", "Questions"].map((s, i) => (
            <div
              key={i}
              className={`flex-1 text-center ${
                step === i + 1 ? "text-blue-500" : "text-gray-400"
              }`}
            >
              {s}
            </div>
          ))}
        </div>

        {step === 1 && <DownloadStep videoId={videoId} setStep={setStep} />}
        {step === 2 && <FramesStep videoId={videoId} setStep={setStep} />}
        {step === 3 && <TagsStep videoId={videoId} setStep={setStep} />}
        {step === 4 && <QuestionsStep videoId={videoId} kidId={kidId} />}
      </div>
    </div>
  );
}

import { useState } from "react";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function QuestionsStep({ videoId, kidId }) {
  const [questions, setQuestions] = useState([]);
  const [start, setStart] = useState(0);
  const [end, setEnd] = useState(10);
  const [loading, setLoading] = useState(false);
  const [assigning, setAssigning] = useState(false);

  function formatTime(sec) {
    const m = Math.floor(sec / 60);
    const s = sec % 60;
    return `${m}:${s < 10 ? "0" : ""}${s}`;
  }

  async function handleGenerate() {
    setLoading(true);

    const res = await fetch(
      `${BASE_URL}/api/openai/${videoId}?start=${start}&end=${end}`,
    );

    const data = await res.json();
    setQuestions(data.questions);

    setLoading(false);
  }

  async function handleAssign() {
    setAssigning(true);

    await fetch(`${BASE_URL}/api/kids/${kidId}/videos_assigned`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ video_id: videoId }),
    });

    setAssigning(false);
    alert("✅ Assigned!");
  }

  return (
    <div>
      <div className="mb-4">
        <input
          type="range"
          min="0"
          max="120"
          value={start}
          onChange={(e) => setStart(Number(e.target.value))}
          className="w-full"
        />
        <input
          type="range"
          min="0"
          max="120"
          value={end}
          onChange={(e) => setEnd(Number(e.target.value))}
          className="w-full"
        />

        <div className="flex justify-between text-sm text-gray-600">
          <span>Start: {formatTime(start)}</span>
          <span>End: {formatTime(end)}</span>
        </div>
      </div>

      <button
        onClick={handleGenerate}
        className="w-full py-3 bg-yellow-500 text-white rounded-xl mb-4"
      >
        {loading ? "Generating..." : "Generate Questions 🤖"}
      </button>

      <div className="space-y-3">
        {questions.map((q, i) => (
          <div key={i} className="p-4 border rounded-xl">
            <p className="font-semibold">❓ {q.question}</p>
            <p className="text-sm text-gray-600">Answer: {q.answer}</p>
          </div>
        ))}
      </div>

      {questions.length > 0 && (
        <button
          onClick={handleAssign}
          className="w-full mt-4 py-3 bg-green-500 text-white rounded-xl"
        >
          {assigning ? "Assigning..." : "✅ Assign to Kid"}
        </button>
      )}
    </div>
  );
}

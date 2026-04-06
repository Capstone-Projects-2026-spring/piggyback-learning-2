"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function QuestionsStep({ videoId, kidId }) {
  const router = useRouter();

  const [questions, setQuestions] = useState([]);
  const [start, setStart] = useState(0);
  const [end, setEnd] = useState(10);
  const [interval, setInterval] = useState(10);
  const [loading, setLoading] = useState(false);
  const [assigning, setAssigning] = useState(false);

  const iconForType = {
    action: "🚀",
    causal: "🔗",
    character: "👤",
    feeling: "❤️",
    outcome: "🏁",
    prediction: "🔮",
    setting: "📍",
  };

  async function handleGenerate() {
    setLoading(true);

    const startPoints = [];
    for (let t = start; t <= end; t += interval) {
      startPoints.push(t);
    }

    try {
      const responses = await Promise.all(
        startPoints.map(async (s) => {
          const e = Math.min(s + interval, end);
          if (s == e) return;
          return fetch(
            `${BASE_URL}/api/openai/${videoId}?start=${s}&end=${e}`,
          ).then((res) => res.json());
        }),
      );

      // responses is an array of objects like { segment, questions }
      const merged = responses.flatMap((r) =>
        (r?.questions || []).map((q) => ({
          ...q,
          segment: r.segment,
        })),
      );

      setQuestions(merged);
    } catch (error) {
      console.error(error);
      alert("⚠️ Error generating questions");
    }

    setLoading(false);
  }

  // group by segment id
  const grouped = questions.reduce((acc, q) => {
    const id = q.segment.id;
    if (!acc[id]) acc[id] = { segment: q.segment, questions: [] };
    acc[id].questions.push(q);
    return acc;
  }, {});

  async function handleAssign() {
    setAssigning(true);

    await fetch(`${BASE_URL}/api/kids/${kidId}/videos_assigned`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ video_id: videoId }),
    });

    setAssigning(false);
    router.push(`/kids/${kidId}`);
  }

  return (
    <div className="space-y-6">
      {/* INPUTS */}
      <div className="grid grid-cols-3 gap-4">
        <div>
          <label className="block text-sm font-semibold text-gray-800">
            Start (sec)
          </label>
          <input
            type="number"
            min="0"
            value={start}
            onChange={(e) => setStart(Number(e.target.value))}
            className="w-full border border-blue-300 rounded-xl px-3 py-2 text-gray-800"
          />
        </div>

        <div>
          <label className="block text-sm font-semibold text-gray-800">
            End (sec)
          </label>
          <input
            type="number"
            min="0"
            value={end}
            onChange={(e) => setEnd(Number(e.target.value))}
            className="w-full border border-blue-300 rounded-xl px-3 py-2 text-gray-800"
          />
        </div>

        <div>
          <label className="block text-sm font-semibold text-gray-800">
            Interval (sec)
          </label>
          <input
            type="number"
            min="1"
            value={interval}
            onChange={(e) => setInterval(Number(e.target.value))}
            className="w-full border border-blue-300 rounded-xl px-3 py-2 text-gray-800"
          />
        </div>
      </div>

      {/* GENERATE BUTTON */}
      <button
        onClick={handleGenerate}
        className="w-full py-3 bg-yellow-500 text-white font-bold rounded-xl hover:bg-yellow-600 transition"
      >
        {loading ? "Generating..." : "Generate Questions 🤖"}
      </button>

      {/* GROUPED QUESTIONS */}
      <div className="space-y-8">
        {Object.values(grouped).map(({ segment, questions }) => (
          <div
            key={segment.id}
            className="bg-linear-to-br from-blue-50 via-blue-100 to-blue-50 p-4 rounded-2xl shadow-md border border-blue-200"
          >
            {/* Segment Header */}
            <div className="flex justify-between items-center mb-3">
              <h2 className="text-lg font-bold text-blue-700">
                Best question: {segment.best_question}
              </h2>
              <span className="text-sm text-gray-600">
                {segment.start_seconds}–{segment.end_seconds} sec
              </span>
            </div>

            <div className="space-y-3">
              {questions.map((q, i) => (
                <div
                  key={i}
                  className="flex gap-3 items-start bg-white p-3 rounded-xl shadow"
                >
                  <div className="text-2xl">{iconForType[q.qtype] ?? "❓"}</div>
                  <div className="grow">
                    <p className="font-semibold text-gray-800">{q.question}</p>
                    <p className="text-sm text-gray-600 mt-1">
                      Answer: {q.answer}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>

      {questions.length > 0 && (
        <button
          onClick={handleAssign}
          className="w-full mt-4 py-3 bg-green-500 text-white font-bold rounded-xl hover:bg-green-600 transition"
        >
          {assigning ? "Assigning..." : "✅ Assign to Kid"}
        </button>
      )}
    </div>
  );
}

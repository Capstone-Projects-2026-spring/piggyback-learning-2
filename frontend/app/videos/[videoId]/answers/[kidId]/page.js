"use client";

import { useParams } from "next/navigation";
import { useEffect, useState } from "react";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function ResultsPage() {
  const params = useParams();
  const { videoId, kidId } = params;

  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchResults() {
      try {
        const res = await fetch(`${BASE_URL}/api/answers/${kidId}/${videoId}`);
        const json = await res.json();
        setData(json);
      } catch (err) {
        console.error("Error fetching results:", err);
      } finally {
        setLoading(false);
      }
    }

    fetchResults();
  }, [videoId, kidId]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen text-xl font-bold">
        Loading results...
      </div>
    );
  }

  if (!data) {
    return <div className="p-6">No results found.</div>;
  }

  const total = data.answers.length;
  const correct = data.answers.filter((a) => a.is_correct).length;
  const accuracy = total ? Math.round((correct / total) * 100) : 0;

  const grouped = {};
  data.answers.forEach((ans) => {
    if (!grouped[ans.segment_id]) {
      grouped[ans.segment_id] = [];
    }
    grouped[ans.segment_id].push(ans);
  });

  return (
    <div className="min-h-screen bg-linear-to-br from-yellow-100 to-pink-100 p-6">
      <h1 className="text-3xl font-bold text-center mb-6 text-purple-700">
        🎉 Results
      </h1>

      <div className="bg-white rounded-2xl shadow p-6 mb-6 text-center">
        <p className="text-lg font-semibold text-gray-800">
          Total Answers: <span className="text-blue-600">{total}</span>
        </p>
        <p className="text-lg font-semibold text-gray-800">
          Correct: <span className="text-green-600">{correct}</span>
        </p>
        <p className="text-lg font-semibold text-gray-800">
          Accuracy: <span className="text-purple-600">{accuracy}%</span>
        </p>
      </div>

      <div className="space-y-6">
        {Object.entries(grouped).map(([segmentId, answers]) => (
          <div key={segmentId} className="bg-white rounded-2xl shadow p-5">
            <h2 className="text-xl font-bold text-indigo-600 mb-4">
              Segment {segmentId}
            </h2>

            <div className="space-y-3">
              {answers.map((ans, idx) => (
                <div
                  key={idx}
                  className={`p-3 rounded-xl flex flex-col md:flex-row md:items-center md:justify-between ${
                    ans.is_correct ? "bg-green-100" : "bg-red-100"
                  }`}
                >
                  <div>
                    <p className="font-semibold text-gray-800">
                      🗣️ {ans.transcript || "No answer"}
                    </p>
                    <p className="text-sm text-gray-600">Mood: {ans.mood}</p>
                  </div>

                  <div className="mt-2 md:mt-0 text-right">
                    <p className="text-sm text-gray-800">
                      Similarity: {(ans.similarity_score * 100).toFixed(1)}%
                    </p>
                    <p className="font-bold text-gray-800">
                      {ans.is_correct ? "✅ Correct" : "❌ Wrong"}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

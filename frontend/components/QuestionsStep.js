"use client";

import { useRouter } from "next/navigation";
import { useState, useCallback } from "react";
import { usePiggy } from "@/context/PiggyContext";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function QuestionsStep({ videoId, kidId }) {
  const router = useRouter();

  const [questions, setQuestions] = useState([]);
  const [bestQuestions, setBestQuestions] = useState({});
  const [start, setStart] = useState(0);
  const [end, setEnd] = useState(10);
  const [interval, setInterval] = useState(10);
  const [loading, setLoading] = useState(false);
  const [assigning, setAssigning] = useState(false);
  const [saving, setSaving] = useState({});
  const [expandedFollowups, setExpandedFollowups] = useState({});
  const { setPiggyText } = usePiggy();

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
    setPiggyText("Creating questions... 🧠");
    setLoading(true);

    const startPoints = [];
    for (let t = start; t <= end; t += interval) {
      startPoints.push(t);
    }

    try {
      // First trigger generation for each segment
      await Promise.all(
        startPoints.map(async (s) => {
          const e = Math.min(s + interval, end);
          if (s == e) return;
          return fetch(`${BASE_URL}/api/openai/${videoId}?start=${s}&end=${e}`);
        }),
      );

      // Then fetch the full question objects (with ids and followup data) from the questions endpoint
      const res = await fetch(`${BASE_URL}/api/questions/${videoId}`);
      const data = await res.json();

      const initialBestQuestions = {};
      data.segments.forEach((seg) => {
        initialBestQuestions[seg.id] = seg.best_question;
      });
      setBestQuestions(initialBestQuestions);

      const merged = data.segments.flatMap((seg) =>
        seg.questions.map((q) => ({
          ...q,
          segment: {
            id: seg.id,
            start_seconds: seg.start_seconds,
            end_seconds: seg.end_seconds,
            best_question: seg.best_question,
          },
          _question: q.question,
          _answer: q.answer,
          _followup_enabled: q.followup_enabled ?? false,
          _followup_correct_question: q.followup_for_correct_answer?.question ?? "",
          _followup_correct_answer: q.followup_for_correct_answer?.answer ?? "",
          _followup_wrong_question: q.followup_for_wrong_answer?.question ?? "",
          _followup_wrong_answer: q.followup_for_wrong_answer?.answer ?? "",
        }))
      );

      setQuestions(merged);

    } catch (error) {
      console.error(error);
      alert("⚠️ Error generating questions");
    }

    setLoading(false);
  }

  function updateQuestion(questionId, field, value) {
    setQuestions((prev) =>
      prev.map((q) => (q.id === questionId ? { ...q, [field]: value } : q))
    );
  }

  async function saveQuestion(q) {
    setSaving((prev) => ({ ...prev, [q.id]: true }));
    try {
      await fetch(`${BASE_URL}/api/questions/${q.id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          question: q._question,
          answer: q._answer,
          followup_enabled: q._followup_enabled,
          followup_correct_question: q._followup_correct_question,
          followup_correct_answer: q._followup_correct_answer,
          followup_wrong_question: q._followup_wrong_question,
          followup_wrong_answer: q._followup_wrong_answer,
        }),
      });
    } catch (err) {
      console.error("Failed to save question", err);
      alert("⚠️ Failed to save question");
    }
    setSaving((prev) => ({ ...prev, [q.id]: false }));
  }

  async function saveSegmentBestQuestion(segmentId, bestQuestion) {
    try {
      await fetch(`${BASE_URL}/api/questions/segment/${segmentId}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ best_question: bestQuestion }),
      });
    } catch (err) {
      console.error("Failed to save best question", err);
    }
  }

  function handleKeyDown(e, q) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      saveQuestion(q);
    }
  }

  function toggleFollowup(questionId) {
    setExpandedFollowups((prev) => ({
      ...prev,
      [questionId]: !prev[questionId],
    }));
  }

  // group by segment id
  const grouped = questions.reduce((acc, q) => {
    const id = q.segment.id;
    if (!acc[id]) acc[id] = { segment: q.segment, questions: [], _best_question: q.segment.best_question };
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
          <label className="block text-sm font-semibold text-gray-800">Start (sec)</label>
          <input
            type="number"
            min="0"
            value={start}
            onChange={(e) => setStart(Number(e.target.value))}
            className="w-full border border-blue-300 rounded-xl px-3 py-2 text-gray-800"
          />
        </div>
        <div>
          <label className="block text-sm font-semibold text-gray-800">End (sec)</label>
          <input
            type="number"
            min="0"
            value={end}
            onChange={(e) => setEnd(Number(e.target.value))}
            className="w-full border border-blue-300 rounded-xl px-3 py-2 text-gray-800"
          />
        </div>
        <div>
          <label className="block text-sm font-semibold text-gray-800">Interval (sec)</label>
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
        {Object.values(grouped).map(({ segment, questions: segQuestions, _best_question }) => (
          <div
            key={segment.id}
            className="bg-linear-to-br from-blue-50 via-blue-100 to-blue-50 p-4 rounded-2xl shadow-md border border-blue-200"
          >
            {/* Segment Header */}
            <div className="flex justify-between items-center mb-3">
              <div className="flex items-center gap-2">
                <span className="text-sm font-semibold text-blue-700">Best question:</span>
                <select
                  className="text-sm border border-blue-300 rounded-lg px-2 py-1 text-blue-700 bg-white"
                  value={bestQuestions[segment.id] ?? ""}
                  onChange={(e) => {
                    const val = e.target.value;
                    setBestQuestions((prev) => ({ ...prev, [segment.id]: val }));
                    saveSegmentBestQuestion(segment.id, val);
                  }}
                >
                  {segQuestions.map((q) => (
                    <option key={q.id} value={q._question}>
                      {q._question}
                    </option>
                  ))}
                </select>
              </div>
              <span className="text-sm text-gray-600">
                {segment.start_seconds}–{segment.end_seconds} sec
              </span>
            </div>

            <div className="space-y-3">
              {segQuestions.map((q) => (
                <div
                  key={q.id}
                  className="flex gap-3 items-start bg-white p-3 rounded-xl shadow"
                >
                  <div className="text-2xl">{iconForType[q.qtype] ?? "❓"}</div>
                  <div className="grow space-y-2">
                    {/* Question */}
                    <input
                      className="w-full font-semibold text-gray-800 border-b border-gray-200 focus:border-blue-400 outline-none bg-transparent py-1"
                      value={q._question}
                      onChange={(e) => updateQuestion(q.id, "_question", e.target.value)}
                      onKeyDown={(e) => handleKeyDown(e, q)}
                      placeholder="Question"
                    />
                    {/* Answer */}
                    <input
                      className="w-full text-sm text-gray-600 border-b border-gray-200 focus:border-blue-400 outline-none bg-transparent py-1"
                      value={q._answer}
                      onChange={(e) => updateQuestion(q.id, "_answer", e.target.value)}
                      onKeyDown={(e) => handleKeyDown(e, q)}
                      placeholder="Answer"
                    />
                    {/* Followup toggle */}
                    <div className="flex items-center gap-2 pt-1">
                      <label className="flex items-center gap-2 text-sm text-gray-500 cursor-pointer select-none">
                        <input
                          type="checkbox"
                          checked={q._followup_enabled}
                          onChange={(e) => {
                            console.log("toggling followup for q.id:", q.id);
                            updateQuestion(q.id, "_followup_enabled", e.target.checked);
                          }}
                          
                          className="rounded"
                        />
                        Include followup
                      </label>
                      {q._followup_enabled && (
                        <button
                          onClick={() => toggleFollowup(q.id)}
                          className="text-xs text-blue-500 hover:underline"
                        >
                          {expandedFollowups[q.id] ? "▲" : "▼"}
                        </button>
                      )}
                    </div>

                    {/* Followup fields */}
                    {q._followup_enabled && expandedFollowups[q.id] && (
                      <div className="mt-2 space-y-2 pl-3 border-l-2 border-blue-100">
                        <p className="text-xs font-semibold text-green-600">✅ If correct:</p>
                        <input
                          className="w-full text-sm text-gray-700 border-b border-gray-200 focus:border-green-400 outline-none bg-transparent py-1"
                          value={q._followup_correct_question}
                          onChange={(e) => updateQuestion(q.id, "_followup_correct_question", e.target.value)}
                          onKeyDown={(e) => handleKeyDown(e, q)}
                          placeholder="Followup question"
                        />
                        <input
                          className="w-full text-sm text-gray-600 border-b border-gray-200 focus:border-green-400 outline-none bg-transparent py-1"
                          value={q._followup_correct_answer}
                          onChange={(e) => updateQuestion(q.id, "_followup_correct_answer", e.target.value)}
                          onKeyDown={(e) => handleKeyDown(e, q)}
                          placeholder="Followup answer"
                        />
                        <p className="text-xs font-semibold text-red-500 pt-1">❌ If wrong:</p>
                        <input
                          className="w-full text-sm text-gray-700 border-b border-gray-200 focus:border-red-300 outline-none bg-transparent py-1"
                          value={q._followup_wrong_question}
                          onChange={(e) => updateQuestion(q.id, "_followup_wrong_question", e.target.value)}
                          onKeyDown={(e) => handleKeyDown(e, q)}
                          placeholder="Followup question"
                        />
                        <input
                          className="w-full text-sm text-gray-600 border-b border-gray-200 focus:border-red-300 outline-none bg-transparent py-1"
                          value={q._followup_wrong_answer}
                          onChange={(e) => updateQuestion(q.id, "_followup_wrong_answer", e.target.value)}
                          onKeyDown={(e) => handleKeyDown(e, q)}
                          placeholder="Followup answer"
                        />
                      </div>
                    )}

                    {/* Save button */}
                    <button
                      onClick={() => saveQuestion(q)}
                      disabled={saving[q.id]}
                      className="mt-1 text-xs px-3 py-1 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition disabled:opacity-50"
                    >
                      {saving[q.id] ? "Saving..." : "Save"}
                    </button>
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
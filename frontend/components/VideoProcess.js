"use client";

import { useState, useEffect } from "react";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

function formatTime(sec) {
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${m}:${s < 10 ? "0" : ""}${s}`;
}

export default function VideoProcess({ videoId, kidId }) {
  const [step, setStep] = useState(1);
  const [loading, setLoading] = useState(false);

  const [tags, setTags] = useState([]);
  const [selectedTags, setSelectedTags] = useState([]);
  const [newTag, setNewTag] = useState("");

  const [questions, setQuestions] = useState([]);

  const [start, setStart] = useState(0);
  const [end, setEnd] = useState(10);

  const [assigning, setAssigning] = useState(false);

  // -----------------------------
  // Fetch tags
  // -----------------------------
  useEffect(() => {
    fetch(`${BASE_URL}/api/tags`)
      .then((res) => res.json())
      .then(setTags);
  }, []);

  // -----------------------------
  // STEP 1: Download
  // -----------------------------
  async function handleDownload() {
    setLoading(true);

    const res = await fetch(`${BASE_URL}/api/videos/download/${videoId}`);
    const data = await res.json();

    setLoading(false);

    // ✅ AUTO SKIP
    if (data.success || data.msg?.includes("already")) {
      setStep(3);
    } else {
      setStep(2);
    }
  }

  // -----------------------------
  // STEP 2: Extract Frames
  // -----------------------------
  async function handleExtract() {
    setLoading(true);

    const res = await fetch(`${BASE_URL}/api/frames/extract/${videoId}`);
    const data = await res.json();

    setLoading(false);

    if (data.success) setStep(3);
  }

  // -----------------------------
  // STEP 3: Tags
  // -----------------------------
  function toggleTag(id) {
    setSelectedTags((prev) =>
      prev.includes(id) ? prev.filter((t) => t !== id) : [...prev, id],
    );
  }

  async function handleCreateTag() {
    if (!newTag.trim()) return;

    const res = await fetch(`${BASE_URL}/api/tags`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ name: newTag }),
    });

    const created = await res.json();

    setTags((prev) => [...prev, created]);
    setNewTag("");
  }

  async function handleSaveTags() {
    setLoading(true);

    await fetch(`${BASE_URL}/api/videos/${videoId}/tags`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        tags: selectedTags, // ✅ FIXED
      }),
    });

    setLoading(false);
    setStep(4);
  }

  // -----------------------------
  // STEP 4: Generate Questions
  // -----------------------------
  async function handleGenerate() {
    setLoading(true);

    const res = await fetch(
      `${BASE_URL}/api/openai/${videoId}?start=${start}&end=${end}`,
    );

    const data = await res.json();
    setQuestions(data.questions);

    setLoading(false);
  }

  // -----------------------------
  // Assign Video
  // -----------------------------
  async function handleAssign() {
    setAssigning(true);

    await fetch(`${BASE_URL}/api/kids/${kidId}/videos_assigned`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        video_id: videoId,
      }),
    });

    setAssigning(false);
    alert("✅ Assigned!");
  }

  // -----------------------------
  // UI
  // -----------------------------
  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-purple-50 to-pink-50 p-6">
      <div className="max-w-3xl mx-auto bg-white rounded-3xl shadow-xl p-6">
        <h1 className="text-2xl font-bold mb-6 text-gray-800">
          🎬 Process Video
        </h1>

        {/* Steps */}
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

        {/* STEP 1 */}
        {step === 1 && (
          <button
            onClick={handleDownload}
            className="w-full py-3 bg-blue-500 text-white rounded-xl"
          >
            {loading ? "Downloading..." : "Download Video 📥"}
          </button>
        )}

        {/* STEP 2 */}
        {step === 2 && (
          <button
            onClick={handleExtract}
            className="w-full py-3 bg-purple-500 text-white rounded-xl"
          >
            {loading ? "Extracting..." : "Extract Frames 🎞️"}
          </button>
        )}

        {/* STEP 3 */}
        {step === 3 && (
          <div>
            <div className="flex flex-wrap gap-2 mb-4">
              {tags.map((tag) => (
                <button
                  key={tag.id}
                  onClick={() => toggleTag(tag.id)}
                  className={`px-3 py-1 rounded-full ${
                    selectedTags.includes(tag.id)
                      ? "bg-green-500 text-white"
                      : "bg-gray-200 text-gray-800"
                  }`}
                >
                  {tag.name}
                </button>
              ))}
            </div>

            {/* CREATE TAG */}
            <div className="flex gap-2 mb-4">
              <input
                value={newTag}
                onChange={(e) => setNewTag(e.target.value)}
                placeholder="New tag..."
                className="flex-grow border border-gray-300 bg-white text-gray-800 rounded-xl px-3 py-2"
              />
              <button
                onClick={handleCreateTag}
                className="bg-blue-500 text-white px-4 rounded-xl"
              >
                Add
              </button>
            </div>

            <button
              onClick={handleSaveTags}
              className="w-full py-3 bg-green-500 text-white rounded-xl"
            >
              {loading ? "Saving..." : "Save Tags 🏷️"}
            </button>
          </div>
        )}

        {/* STEP 4 */}
        {step === 4 && (
          <div>
            {/* TIME RANGE */}
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

            {/* QUESTIONS */}
            <div className="space-y-3">
              {questions.map((q, i) => (
                <div
                  key={i}
                  className="bg-white border border-gray-200 p-4 rounded-xl shadow-sm"
                >
                  <p className="font-semibold text-gray-800">❓ {q.question}</p>
                  <p className="text-sm text-gray-600">Answer: {q.answer}</p>
                </div>
              ))}
            </div>

            {/* ASSIGN BUTTON */}
            {questions.length > 0 && (
              <button
                onClick={handleAssign}
                className="w-full mt-4 py-3 bg-green-500 text-white rounded-xl"
              >
                {assigning ? "Assigning..." : "✅ Assign to Kid"}
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

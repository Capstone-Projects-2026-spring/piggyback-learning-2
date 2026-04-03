import { useEffect, useState } from "react";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function TagsStep({ videoId, setStep }) {
  const [tags, setTags] = useState([]);
  const [selectedTags, setSelectedTags] = useState([]);
  const [newTag, setNewTag] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    fetch(`${BASE_URL}/api/tags`)
      .then((res) => res.json())
      .then(setTags);
  }, []);

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
        tags: selectedTags,
      }),
    });

    setLoading(false);
    setStep(4);
  }

  return (
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

      <div className="flex gap-2 mb-4">
        <input
          value={newTag}
          onChange={(e) => setNewTag(e.target.value)}
          placeholder="New tag..."
          className="grow border border-gray-300 rounded-xl px-3 py-2"
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
  );
}

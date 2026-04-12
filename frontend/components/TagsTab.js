"use client";

import { useEffect, useState } from "react";
import { usePiggy } from "@/context/PiggyContext";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function TagsTab({ kidId, onTagsUpdated }) {
  const [allTags, setAllTags] = useState([]);
  const [assignedTags, setAssignedTags] = useState([]);
  const [newTag, setNewTag] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const { setPiggyText, setPiggyMode } = usePiggy();

  // Fetch all tags and kid's assigned tags
  useEffect(() => {
    async function fetchTags() {
      try {
        const [allRes, assignedRes] = await Promise.all([
          fetch(`${BASE_URL}/api/tags`),
          fetch(`${BASE_URL}/api/kids/${kidId}/tags`),
        ]);
        const allData = await allRes.json();
        const assignedData = await assignedRes.json();
        setAllTags(allData || []);
        setAssignedTags(assignedData || []);
      } catch (err) {
        console.error("Failed to fetch tags", err);
      } finally {
        setLoading(false);
      }
    }

    fetchTags();
  }, [kidId]);

  async function handleCreateTag(e) {
    e.preventDefault();
    if (!newTag.trim()) return;

    try {
      const res = await fetch(`${BASE_URL}/api/tags`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: newTag.trim() }),
      });
      const created = await res.json();
      setAllTags((prev) => [...prev, created]);
      setNewTag("");
      setPiggyMode("tags");
      setPiggyText("Nice! Now select the tag and assign it.");
    } catch (err) {
      console.error("Failed to create tag", err);
    }
  }

  // Assign tags + trigger refresh
  async function handleAssignTags() {
    if (assignedTags.length === 0) return;

    setSaving(true);
    try {
      await fetch(`${BASE_URL}/api/kids/${kidId}/tags`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ tags: assignedTags.map((t) => t.id) }),
      });

      // Refresh recommendations in parent
      if (onTagsUpdated) {
        await onTagsUpdated();
      }
      setPiggyMode("recommended");
      setPiggyText("Nice! Your tags are ready. Check Recommended.");


      //alert("Tags assigned successfully ✅");
    } catch (err) {
      console.error("Failed to assign tags", err);
      alert("Failed to assign tags ❌");
    } finally {
      setSaving(false);
    }
  }

  if (loading) return <p>Loading tags... 🏷️</p>;

  return (
    <div className="bg-white p-6 rounded-2xl shadow border">
      {/* Create tag */}
      <form onSubmit={handleCreateTag} className="flex mb-4">
        <input
          type="text"
          value={newTag}
          onChange={(e) => setNewTag(e.target.value)}
          onFocus={() => {
            setPiggyMode("tags");
            setPiggyText("Add a tag to get better video recommendations!");
          }}
          placeholder="Create new tag..."
          className="grow border border-gray-300 px-4 py-2 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-400 text-gray-800"
        />
        <button className="ml-3 bg-blue-500 text-white px-4 py-2 rounded-xl hover:bg-blue-600">
          Add
        </button>
      </form>

      {/* Tag list */}
      <div className="flex flex-wrap gap-2 mb-4">
        {allTags.map((tag) => {
          const isSelected = assignedTags.some((t) => t.id === tag.id);
          return (
            <button
              key={tag.id}
              onClick={() => {
                if (isSelected) {
                  setAssignedTags((prev) =>
                    prev.filter((t) => t.id !== tag.id),
                  );
                } else {
                  setAssignedTags((prev) => [...prev, tag]);
                }
              }}
              className={`px-4 py-2 rounded-xl font-semibold border ${
                isSelected
                  ? "bg-green-400 text-white border-green-500"
                  : "bg-gray-100 text-gray-800 border-gray-300 hover:bg-gray-200"
              }`}
            >
              {tag.name}
            </button>
          );
        })}
      </div>

      <button
        onClick={handleAssignTags}
        disabled={saving}
        className="bg-purple-500 text-white px-6 py-2 rounded-xl hover:bg-purple-600"
      >
        {saving ? "Saving..." : "Assign Selected Tags"}
      </button>
    </div>
  );
}

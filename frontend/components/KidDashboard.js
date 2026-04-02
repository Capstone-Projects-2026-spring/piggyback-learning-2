"use client";

import { AuthContext } from "@/app/context/AuthContext";
import Image from "next/image";
import { useContext, useEffect, useState } from "react";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

function formatDuration(sec) {
  const minutes = Math.floor(sec / 60);
  const seconds = sec % 60;
  return `${minutes}:${seconds < 10 ? "0" : ""}${seconds}`;
}

export default function KidDashboard({ kidId }) {
  const { role } = useContext(AuthContext);

  const [assigned, setAssigned] = useState([]);
  const [recommended, setRecommended] = useState([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState("assigned");

  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState([]);
  const [searchLoading, setSearchLoading] = useState(false);

  useEffect(() => {
    if (!kidId) return;

    async function fetchData() {
      try {
        const [assignedRes, recRes] = await Promise.all([
          fetch(`${BASE_URL}/api/kids/${kidId}/videos_assigned`),
          fetch(`${BASE_URL}/api/kids/${kidId}/recommendations`),
        ]);

        const assignedData = await assignedRes.json();
        const recData = await recRes.json();

        let videos = recData.recommendations || [];
        const tags = recData.tags || [];

        if (videos.length < 10 && tags.length > 0) {
          for (const tag of tags) {
            const searchRes = await fetch(
              `/api/search?q=${encodeURIComponent(tag)}`,
            );
            const { videos: ytVideos } = await searchRes.json();

            const formatted = ytVideos.slice(0, 3).map((v) => ({
              id: v.id,
              title: v.title,
              thumbnail_url: v.thumbnail,
              duration: formatDuration(v.seconds),
              score: "N/A",
            }));

            videos = videos.concat(formatted);
            if (videos.length >= 10) break;
          }
          videos = videos.slice(0, 10);
        }

        setAssigned(assignedData);
        setRecommended(videos);
      } catch (err) {
        console.error("Failed to load dashboard", err);
      } finally {
        setLoading(false);
      }
    }

    fetchData();
  }, [kidId]);

  async function handleSearch(e) {
    e.preventDefault();
    if (!searchQuery.trim()) return;

    setSearchLoading(true);
    try {
      const res = await fetch(
        `/api/search?q=${encodeURIComponent(searchQuery)}`,
      );
      const { videos } = await res.json();
      setSearchResults(
        videos.slice(0, 10).map((v) => ({
          id: v.id,
          title: v.title,
          thumbnail_url: v.thumbnail,
          duration: formatDuration(v.seconds),
        })),
      );
      setActiveTab("search"); // switch to search results
    } catch (err) {
      console.error("Search failed", err);
    } finally {
      setSearchLoading(false);
    }
  }

  if (loading) {
    return <p className="text-center mt-10">Loading videos... 🎬</p>;
  }

  // Decide which list we’re showing
  const videos =
    activeTab === "assigned"
      ? assigned
      : activeTab === "recommended"
        ? recommended
        : searchResults;

  return (
    <div>
      {/* Search Bar */}
      {role === "parent" && (
        <form onSubmit={handleSearch} className="mb-6 flex">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search YouTube..."
            className="
              grow
              border border-gray-300
              bg-white text-gray-800
              focus:outline-none focus:ring-2 focus:ring-blue-400
              rounded-xl px-4 py-2
              dark:bg-gray-800 dark:text-gray-100
            "
          />
          <button
            type="submit"
            className="ml-3 bg-blue-500 text-white px-4 py-2 rounded-xl hover:bg-blue-600"
          >
            {searchLoading ? "Searching..." : "Search"}
          </button>
        </form>
      )}

      {/* Tabs */}
      <div className="flex mb-6 bg-white rounded-2xl p-1 shadow border border-gray-200 w-fit">
        <button
          onClick={() => setActiveTab("assigned")}
          className={`px-5 py-2 rounded-xl font-semibold ${
            activeTab === "assigned"
              ? "bg-linear-to-r from-blue-400 to-purple-400 text-white shadow"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          📚 Assigned
        </button>

        <button
          onClick={() => setActiveTab("recommended")}
          className={`px-5 py-2 rounded-xl font-semibold ${
            activeTab === "recommended"
              ? "bg-linear-to-r from-green-400 to-blue-400 text-white shadow"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          🤖 Recommended
        </button>

        <button
          onClick={() => setActiveTab("search")}
          className={`px-5 py-2 rounded-xl font-semibold ${
            activeTab === "search"
              ? "bg-linear-to-r from-yellow-400 to-red-400 text-white shadow"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          🔍 Search
        </button>
      </div>

      {videos.length === 0 ? (
        <div className="text-center bg-white p-8 rounded-2xl shadow border">
          <p className="text-gray-600 text-lg">
            {activeTab === "assigned"
              ? "No assigned videos yet 📭"
              : activeTab === "recommended"
                ? "No recommendations yet 🤖"
                : "No search results 🤔"}
          </p>
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2">
          {videos.map((video) => (
            <div
              key={video.id}
              className="bg-white rounded-2xl shadow border p-3 hover:scale-105 transition transform"
            >
              <div className="relative w-full h-40">
                <Image
                  src={video.thumbnail_url}
                  alt={video.title}
                  fill
                  className="rounded-xl object-cover"
                />
              </div>

              <p className="font-semibold text-gray-800">{video.title}</p>

              <p className="text-sm text-gray-500">⏱ {video.duration}</p>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

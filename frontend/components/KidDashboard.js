"use client";

import Image from "next/image";
import { useEffect, useState } from "react";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function KidDashboard({ kidId }) {
  const [assigned, setAssigned] = useState([]);
  const [recommended, setRecommended] = useState([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState("assigned");

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

        setAssigned(assignedData);
        setRecommended(recData);
      } catch (err) {
        console.error("Failed to load dashboard", err);
      } finally {
        setLoading(false);
      }
    }

    fetchData();
  }, [kidId]);

  if (loading) {
    return <p className="text-center mt-10">Loading videos... 🎬</p>;
  }

  const videos = activeTab === "assigned" ? assigned : recommended;

  return (
    <div>
      {/* Tabs */}
      <div className="flex mb-6 bg-white rounded-2xl p-1 shadow border border-gray-200 w-fit">
        <button
          onClick={() => setActiveTab("assigned")}
          className={`px-5 py-2 rounded-xl font-semibold transition ${
            activeTab === "assigned"
              ? "bg-linear-to-r from-blue-400 to-purple-400 text-white shadow"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          📚 Assigned
        </button>

        <button
          onClick={() => setActiveTab("recommended")}
          className={`px-5 py-2 rounded-xl font-semibold transition ${
            activeTab === "recommended"
              ? "bg-linear-to-r from-green-400 to-blue-400 text-white shadow"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          🤖 Recommended
        </button>
      </div>

      {/* Content */}
      {videos.length === 0 ? (
        <div className="text-center bg-white p-8 rounded-2xl shadow border">
          <p className="text-gray-600 text-lg">
            {activeTab === "assigned"
              ? "No assigned videos yet 📭"
              : "No recommendations yet 🤖"}
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

              {activeTab === "assigned" ? (
                <p className="text-sm text-gray-500">
                  ⏱ {video.duration_seconds}s
                </p>
              ) : (
                <p className="text-sm text-gray-500">⭐ Score: {video.score}</p>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

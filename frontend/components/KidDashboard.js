"use client";

import { AuthContext } from "@/context/AuthContext";
import { useContext, useEffect, useState } from "react";
import SearchBar from "./SearchBar";
import Tabs from "./Tabs";
import VideoGrid from "./VideoGrid";
import NoVideos from "./NoVideos";
import TagsTab from "./TagsTab";

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
  const [activeTab, setActiveTab] = useState("tags");

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
      setActiveTab("search");
    } catch (err) {
      console.error("Search failed", err);
    } finally {
      setSearchLoading(false);
    }
  }

  if (loading)
    return (
      <p className="text-center text-gray-800 mt-10">Loading videos... 🎬</p>
    );

  const videos =
    activeTab === "assigned"
      ? assigned
      : activeTab === "recommended"
        ? recommended
        : searchResults;

  return (
    <div>
      <SearchBar
        role={role}
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
        onSearch={handleSearch}
        loading={searchLoading}
      />

      <Tabs activeTab={activeTab} setActiveTab={setActiveTab} />

      {activeTab === "tags" ? (
        <TagsTab kidId={kidId} />
      ) : videos.length === 0 ? (
        <NoVideos activeTab={activeTab} />
      ) : (
        <VideoGrid videos={videos} assigned={assigned} kidId={kidId} />
      )}
    </div>
  );
}

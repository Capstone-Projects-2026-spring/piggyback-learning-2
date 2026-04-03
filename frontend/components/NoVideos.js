"use client";

export default function NoVideos({ activeTab }) {
  const messages = {
    assigned: "No assigned videos yet 📭",
    recommended: "No recommendations yet 🤖",
    search: "No search results 🤔",
  };

  return (
    <div className="text-center bg-white p-8 rounded-2xl shadow border">
      <p className="text-gray-600 text-lg">{messages[activeTab]}</p>
    </div>
  );
}

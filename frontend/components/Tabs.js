"use client";

export default function Tabs({ activeTab, setActiveTab, onTabChange }) {
  const tabs = [
    { key: "tags", label: "🏷️ Tags" },
    { key: "assigned", label: "📚 Assigned" },
    { key: "recommended", label: "🤖 Recommended" },
    { key: "search", label: "🔍 Search" },
  ];

  return (
    <div className="flex mb-6 bg-white rounded-2xl p-1 shadow border border-gray-200 w-fit">
      {tabs.map((tab) => (
        <button
          key={tab.key}
          onClick={() => {
            setActiveTab(tab.key);
            if (onTabChange) onTabChange(tab.key);
          }}
          className={`px-5 py-2 rounded-xl font-semibold ${
            activeTab === tab.key
              ? "bg-linear-to-r from-blue-400 to-purple-400 text-white shadow"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}

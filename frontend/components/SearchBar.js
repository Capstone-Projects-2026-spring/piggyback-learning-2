"use client";

export default function SearchBar({
  role,
  searchQuery,
  setSearchQuery,
  onSearch,
  loading,
}) {
  if (role !== "parent") return null;

  return (
    <form onSubmit={onSearch} className="mb-6 flex">
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
        {loading ? "Searching..." : "Search"}
      </button>
    </form>
  );
}

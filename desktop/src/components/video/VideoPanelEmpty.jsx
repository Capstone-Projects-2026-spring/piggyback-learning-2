export default function VideoPanelEmpty({ mode, kidName, searchQuery }) {
  if (searchQuery) {
    return (
      <div className="flex flex-col items-center gap-4 px-8">
        <div className="w-10 h-10 rounded-full border-2 border-pink-200 border-t-pink-400 animate-spin" />
        <p className="text-sm font-medium text-gray-500 text-center">
          Searching for
        </p>
        <p className="text-lg font-bold text-pink-400 text-center">
          "{searchQuery}"
        </p>
        <p className="text-xs text-gray-400 text-center">
          This may take a few seconds…
        </p>
      </div>
    );
  }

  if (mode === "my_videos") {
    return (
      <div className="flex flex-col items-center gap-3 px-8">
        <p className="text-2xl">🎬</p>
        <p className="text-sm text-gray-400 text-center">
          No videos assigned yet.
        </p>
        <p className="text-xs text-gray-300 text-center italic">
          Ask a parent to assign some videos for you
        </p>
      </div>
    );
  }

  if (mode === "recommendations") {
    return (
      <div className="flex flex-col items-center gap-3 px-8">
        <p className="text-2xl">🎯</p>
        <p className="text-sm text-gray-400 text-center">
          No videos found for{" "}
          <span className="text-pink-400 font-medium">{kidName}</span> yet.
        </p>
        <p className="text-xs text-gray-300 text-center italic">
          Try adding more interests first
        </p>
      </div>
    );
  }

  return (
    <p className="text-sm text-gray-400 text-center px-8">
      Say{" "}
      <span className="text-pink-400 font-medium">"search for spiderman"</span>{" "}
      to find videos
    </p>
  );
}

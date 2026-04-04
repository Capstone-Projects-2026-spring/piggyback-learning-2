"use client";

import VideoCard from "./VideoCard";

export default function VideoGrid({ videos, role, assigned, kidId }) {
  return (
    <div className="grid gap-4 sm:grid-cols-2">
      {videos.map((video) => (
        <VideoCard
          key={video.id}
          role={role}
          video={video}
          isAssigned={assigned.some((v) => v.id === video.id)}
          kidId={kidId}
        />
      ))}
    </div>
  );
}

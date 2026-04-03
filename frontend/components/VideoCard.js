"use client";

import Image from "next/image";
import { useRouter } from "next/navigation";

export default function VideoCard({ video, isAssigned, kidId }) {
  const router = useRouter();

  return (
    <div className="bg-white rounded-2xl shadow border p-3 hover:scale-105 transition transform flex flex-col">
      <div className="relative w-full h-40">
        <Image
          src={video.thumbnail_url}
          alt={video.title}
          fill
          sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw"
          className="rounded-xl object-cover"
        />
      </div>

      <p className="font-semibold text-gray-800 mt-2">{video.title}</p>
      <p className="text-sm text-gray-500 mb-3">
        ⏱ {video.duration || video.seconds || video.duration_seconds}s
      </p>

      <button
        onClick={() =>
          router.push(
            isAssigned
              ? `/videos/watch/${video.id}`
              : `/videos/${video.id}/process/${kidId}`,
          )
        }
        className={`mt-auto py-2 rounded-xl font-semibold transition ${
          isAssigned
            ? "bg-green-500 text-white hover:scale-105"
            : "bg-linear-to-r from-purple-400 to-pink-400 text-white hover:scale-105"
        }`}
      >
        {isAssigned ? "▶ Watch" : "➕ Assign"}
      </button>
    </div>
  );
}

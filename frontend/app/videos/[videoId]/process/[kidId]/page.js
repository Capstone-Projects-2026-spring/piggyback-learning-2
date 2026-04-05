"use client";

import ProtectedRoute from "@/components/ProtectedRoute";
import VideoProcess from "@/components/VideoProcess";
import { useParams } from "next/navigation";

export default function VideoPage() {
  const params = useParams();

  return (
    <ProtectedRoute>
      <VideoProcess videoId={params.videoId} kidId={params.kidId} />
    </ProtectedRoute>
  );
}

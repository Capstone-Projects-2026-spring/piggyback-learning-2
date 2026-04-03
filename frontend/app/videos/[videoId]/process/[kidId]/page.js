"use client";

import VideoProcess from "@/components/VideoProcess";
import { useParams } from "next/navigation";

export default function VideoPage() {
  const params = useParams();

  return <VideoProcess videoId={params.videoId} kidId={params.kidId} />;
}

"use client";

import VideoProcess from "@/components/VideoProcess";
import { AuthContext } from "@/context/AuthContext";
import { useParams, useRouter } from "next/navigation";
import { useContext, useEffect } from "react";

export default function VideoPage() {
  const router = useRouter();
  const params = useParams();

  const { isLoggedIn } = useContext(AuthContext);

  useEffect(() => {
    if (!isLoggedIn) router.replace("/login");
  }, [isLoggedIn, router]);

  return <VideoProcess videoId={params.videoId} kidId={params.kidId} />;
}

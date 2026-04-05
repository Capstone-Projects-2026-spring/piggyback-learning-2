"use client";

import KidDashboard from "@/components/KidDashboard";
import { AuthContext } from "@/context/AuthContext";
import { useParams, useRouter } from "next/navigation";
import { useContext, useEffect } from "react";

export default function KidPage() {
  const router = useRouter();

  const params = useParams();
  const kidId = params.id;

  const { isLoggedIn } = useContext(AuthContext);

  useEffect(() => {
    if (!isLoggedIn) router.replace("/login");
  }, [isLoggedIn, router]);

  return (
    <div className="min-h-screen bg-linear-to-br from-blue-100 via-pink-100 to-yellow-100 p-6">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-3xl font-extrabold text-purple-600 mb-6">
          🎬 Kid Dashboard
        </h1>

        <KidDashboard kidId={kidId} />
      </div>
    </div>
  );
}

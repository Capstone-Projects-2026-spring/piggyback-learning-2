"use client";

import KidDashboard from "@/components/KidDashboard";
import ProtectedRoute from "@/components/ProtectedRoute";
import { useParams } from "next/navigation";

export default function KidPage() {
  const params = useParams();
  const kidId = params.id;

  return (
    <ProtectedRoute>
      <div className="min-h-screen bg-linear-to-br from-blue-100 via-pink-100 to-yellow-100 p-6">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-3xl font-extrabold text-purple-600 mb-6">
            🎬 Kid Dashboard
          </h1>

          <KidDashboard kidId={kidId} />
        </div>
      </div>
    </ProtectedRoute>
  );
}

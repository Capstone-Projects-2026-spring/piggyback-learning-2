"use client";

import { useContext, useEffect, useState, useCallback } from "react";
import { useRouter } from "next/navigation";
import { AuthContext } from "@/context/AuthContext";
import KidDashboard from "@/components/KidDashboard";
import ProtectedRoute from "@/components/ProtectedRoute";
import { usePiggy } from "@/context/PiggyContext";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function HomePage() {
  const router = useRouter();

  const [kids, setKids] = useState([]);
  const [name, setName] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);

  const { role, account } = useContext(AuthContext);
  const { setPiggyMode, setPiggyText } = usePiggy();

  const fetchKids = useCallback(async () => {
    if (role !== "parent") return;

    try {
      const res = await fetch(`${BASE_URL}/api/parents/${account.id}/kids`);
      const data = await res.json();
      setKids(data);
    } catch (err) {
      console.error("Failed to fetch kids", err);
    }
  }, [role, account?.id]);

  const handleCreateKid = async (e) => {
    e.preventDefault();
    if (!name || !username || !password) return;

    setLoading(true);
    setPiggyMode("talk");
setPiggyText("Creating the new kid account... ⏳");

    try {
      await fetch(`${BASE_URL}/api/auth/signup`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name,
          username,
          password,
          role: "kid",
          parent_id: account.id,
        }),
      });

      setName("");
      setUsername("");
      setPassword("");
      setModalOpen(false);
      setPiggyText("Yay! New kid added 🎉");
      fetchKids();
    } catch (err) {
      console.error("Failed to create kid", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchKids();
  }, [fetchKids]);

  useEffect(() => {
  if (modalOpen) {
    setPiggyMode("talk");
    setPiggyText("Let’s create a new kid account! 🎉");
  } else {
    setPiggyMode("default");
    setPiggyText("Hi! Let’s get started 🚀");
  }
}, [modalOpen]);

  if (role === "kid") {
    return (
      <ProtectedRoute>
        <div className="min-h-screen bg-linear-to-br from-blue-100 via-pink-100 to-yellow-100 p-6">
          <div className="max-w-4xl mx-auto">
            <h1 className="text-3xl font-extrabold text-purple-600 mb-6">
              🎬 Your Learning Dashboard
            </h1>

            <KidDashboard kidId={account.id} />
          </div>
        </div>
      </ProtectedRoute>
    );
  }

  return (
    <ProtectedRoute>
      <div className="min-h-screen bg-linear-to-br from-blue-100 via-pink-100 to-yellow-100 p-6">
        {/* Header */}
        <div className="max-w-4xl mx-auto mb-8">
          <h1 className="text-3xl font-extrabold text-blue-600">
            👨‍👩‍👧 Your Kids Dashboard
          </h1>
          <p className="text-gray-600 mt-1">
            Manage your kids and track their learning 🎯
          </p>
        </div>

        {/* Kids Grid */}
        <div className="max-w-4xl mx-auto grid gap-4 sm:grid-cols-2">
          {kids.length === 0 ? (
            <div className="col-span-full text-center bg-white p-8 rounded-2xl shadow border border-blue-100">
              <p className="text-lg text-gray-600">🧸 No kids yet!</p>
              <p className="text-sm text-gray-500 mt-1">
                Click the + button to add your first kid 🎉
              </p>
            </div>
          ) : (
            kids.map((kid) => (
              <div
                key={kid.id}
                onClick={() => router.push(`/kids/${kid.id}`)}
                className="cursor-pointer p-5 bg-white rounded-2xl shadow-md border border-blue-100 hover:scale-105 hover:shadow-lg transition transform"
              >
                <p className="text-lg font-bold text-blue-600">👦 {kid.name}</p>
                <p className="text-sm text-gray-500">@{kid.username}</p>
              </div>
            ))
          )}
        </div>

        {/* Floating Add Button */}
        <button
          onClick={() => setModalOpen(true)}
          className="fixed bottom-8 right-8 w-16 h-16 text-3xl text-white rounded-full bg-linear-to-r from-green-400 to-blue-400 shadow-lg hover:scale-110 transition transform"
        >
          +
        </button>

        {/* Modal */}
        {modalOpen && (
          <div className="fixed inset-0 flex items-center justify-center bg-black/40 backdrop-blur-sm z-50">
            <div className="bg-white p-6 rounded-2xl shadow-xl w-full max-w-md relative border border-green-100">
              {/* Close */}
              <button
                onClick={() => setModalOpen(false)}
                className="absolute top-3 right-3 text-gray-400 hover:text-gray-600 text-xl font-bold"
              >
                ×
              </button>

              <h2 className="text-2xl font-bold text-green-500 mb-4">
                🎉 Add a New Kid
              </h2>

              <form onSubmit={handleCreateKid} className="space-y-3">
                <input
                  type="text"
                  placeholder="👤 Name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full rounded-xl border border-green-200 p-3 text-gray-800 placeholder-gray-500 focus:ring-2 focus:ring-green-400 outline-none"
                />

                <input
                  type="text"
                  placeholder="✨ Username"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="w-full rounded-xl border border-green-200 p-3 text-gray-800 placeholder-gray-500 focus:ring-2 focus:ring-green-400 outline-none"
                />

                <input
                  type="password"
                  placeholder="🔒 Password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="w-full rounded-xl border border-green-200 p-3 text-gray-800 placeholder-gray-500 focus:ring-2 focus:ring-green-400 outline-none"
                />

                <button
                  type="submit"
                  disabled={loading}
                  className="w-full rounded-xl bg-linear-to-r from-green-400 to-blue-400 py-3 text-white font-bold hover:scale-105 transition transform disabled:opacity-50"
                >
                  {loading ? "Creating..." : "🚀 Create Kid"}
                </button>
              </form>
            </div>
          </div>
        )}
      </div>
    </ProtectedRoute>
  );
}

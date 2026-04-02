"use client";

import { useContext, useEffect, useState, useCallback } from "react";
import { AuthContext } from "./context/AuthContext";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function HomePage() {
  const [kids, setKids] = useState([]);
  const [name, setName] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);

  const { role, account } = useContext(AuthContext);

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
      fetchKids(); // refresh kids list
    } catch (err) {
      console.error("Failed to create kid", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchKids();
  }, [fetchKids]);

  if (role !== "parent") {
    return (
      <p className="text-center mt-10">
        You must be a parent to view this page.
      </p>
    );
  }

  return (
    <div className="max-w-3xl mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">Your Kids</h1>

      {kids.length === 0 ? (
        <p>No kids yet. Add one below!</p>
      ) : (
        <ul className="mb-6 space-y-2">
          {kids.map((kid) => (
            <li key={kid.id} className="p-2 border rounded shadow-sm">
              <p className="font-semibold">{kid.name}</p>
              <p className="text-sm text-gray-600">{kid.username}</p>
            </li>
          ))}
        </ul>
      )}

      <button
        onClick={() => setModalOpen(true)}
        className="flex items-center justify-center w-16 h-16 text-3xl font-bold text-white bg-blue-600 rounded-full hover:bg-blue-700"
      >
        +
      </button>

      {/* Modal */}
      {modalOpen && (
        <div className="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50 z-50">
          <div className="bg-white p-6 rounded shadow-lg w-full max-w-md relative">
            <button
              onClick={() => setModalOpen(false)}
              className="absolute top-2 right-2 text-gray-500 hover:text-gray-700 font-bold text-xl"
            >
              &times;
            </button>
            <h2 className="text-xl font-bold mb-4">Add a New Kid</h2>
            <form onSubmit={handleCreateKid} className="space-y-2">
              <input
                type="text"
                placeholder="Name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full p-2 border rounded"
              />
              <input
                type="text"
                placeholder="Username"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full p-2 border rounded"
              />
              <input
                type="password"
                placeholder="Password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full p-2 border rounded"
              />
              <button
                type="submit"
                disabled={loading}
                className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
              >
                {loading ? "Creating..." : "Create Kid"}
              </button>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}

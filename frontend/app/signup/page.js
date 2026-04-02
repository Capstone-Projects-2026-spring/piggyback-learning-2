"use client";

import { useState, useContext, useEffect } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { AuthContext } from "../context/AuthContext";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function SignupPage() {
  const router = useRouter();
  const { token } = useContext(AuthContext);

  const [mounted, setMounted] = useState(false);
  const [loading, setLoading] = useState(false);

  const [form, setForm] = useState({
    name: "",
    username: "",
    password: "",
  });

  const [error, setError] = useState("");

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    if (mounted && token) {
      router.replace("/");
    }
  }, [mounted, token, router]);

  async function handleSubmit(e) {
    e.preventDefault();
    setError("");
    setLoading(true);

    try {
      const res = await fetch(`${BASE_URL}/api/auth/signup`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          ...form,
          role: "parent",
          parent_id: null,
        }),
      });

      const data = await res.json();

      if (res.ok) {
        router.push("/login");
      } else {
        setError(data.message || "Signup failed 😢");
      }
    } catch (err) {
      setError("Something went wrong. Try again!");
    }

    setLoading(false);
  }

  if (!mounted) return null;

  return (
    <div className="flex min-h-screen items-center justify-center bg-linear-to-br from-green-100 via-yellow-100 to-pink-100 p-4">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-md space-y-5 rounded-2xl bg-white p-8 shadow-xl border border-green-200"
      >
        <h1 className="text-center text-3xl font-extrabold text-green-500">
          🧸 Create Account
        </h1>

        {error && (
          <p className="text-center text-sm text-red-500 font-medium">
            {error}
          </p>
        )}

        <input
          type="text"
          placeholder="👤 Your Name"
          className="w-full rounded-xl border border-green-200 p-3 text-gray-800 placeholder-gray-500 focus:ring-2 focus:ring-green-400 outline-none"
          value={form.name}
          onChange={(e) => setForm({ ...form, name: e.target.value })}
          required
        />

        <input
          type="text"
          placeholder="✨ Username"
          className="w-full rounded-xl border border-green-200 p-3 text-gray-800 placeholder-gray-500 focus:ring-2 focus:ring-green-400 outline-none"
          value={form.username}
          onChange={(e) => setForm({ ...form, username: e.target.value })}
          required
        />

        <input
          type="password"
          placeholder="🔒 Password"
          className="w-full rounded-xl border border-green-200 p-3 text-gray-800 placeholder-gray-500 focus:ring-2 focus:ring-green-400 outline-none"
          value={form.password}
          onChange={(e) => setForm({ ...form, password: e.target.value })}
          required
        />

        <button
          type="submit"
          disabled={loading}
          className="w-full rounded-xl bg-linear-to-r from-green-400 to-blue-400 py-3 text-white font-bold hover:scale-105 transition transform disabled:opacity-50"
        >
          {loading ? "Creating account..." : "🎉 Sign Up"}
        </button>

        <p className="text-center text-sm text-gray-600">
          Already have an account?{" "}
          <Link
            href="/login"
            className="font-semibold text-green-500 hover:underline"
          >
            Login
          </Link>
        </p>
      </form>
    </div>
  );
}

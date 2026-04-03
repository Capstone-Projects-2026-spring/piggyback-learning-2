"use client";

import { useState, useContext, useEffect } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { AuthContext } from "@/context/AuthContext";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function LoginPage() {
  const router = useRouter();
  const { token, login } = useContext(AuthContext);

  const [mounted, setMounted] = useState(false);
  const [loading, setLoading] = useState(false);

  const [form, setForm] = useState({
    username: "",
    password: "",
    role: "parent",
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
      const res = await fetch(`${BASE_URL}/api/auth/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(form),
      });

      const data = await res.json();

      if (res.ok) {
        login(data.token, data.role, data.account);
        router.push("/");
      } else {
        setError(data.message || "Login failed 😢");
      }
    } catch (err) {
      setError("Something went wrong. Try again!");
    }

    setLoading(false);
  }

  if (!mounted) return null;

  return (
    <div className="flex min-h-screen items-center justify-center bg-linear-to-br from-yellow-100 via-pink-100 to-blue-100 p-4">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-md space-y-5 rounded-2xl bg-white p-8 shadow-xl border border-pink-200"
      >
        <h1 className="text-center text-3xl font-extrabold text-pink-500">
          🎉 Welcome Back!
        </h1>

        {error && (
          <p className="text-center text-sm text-red-500 font-medium">
            {error}
          </p>
        )}

        <select
          className="w-full rounded-xl border border-pink-200 p-3 text-gray-800 focus:ring-2 focus:ring-pink-400 outline-none"
          value={form.role}
          onChange={(e) => setForm({ ...form, role: e.target.value })}
        >
          <option value="parent">👨‍👩‍👧 Parent</option>
          <option value="kid">🧒 Kid</option>
        </select>

        <input
          type="text"
          placeholder="👤 Username"
          className="w-full rounded-xl border border-pink-200 p-3 text-gray-800 placeholder-gray-500 focus:ring-2 focus:ring-pink-400 outline-none"
          value={form.username}
          onChange={(e) => setForm({ ...form, username: e.target.value })}
          required
        />

        <input
          type="password"
          placeholder="🔒 Password"
          className="w-full rounded-xl border border-pink-200 p-3 text-gray-800 placeholder-gray-500 focus:ring-2 focus:ring-pink-400 outline-none"
          value={form.password}
          onChange={(e) => setForm({ ...form, password: e.target.value })}
          required
        />

        <button
          type="submit"
          disabled={loading}
          className="w-full rounded-xl bg-linear-to-r from-pink-400 to-purple-400 py-3 text-white font-bold hover:scale-105 transition transform disabled:opacity-50"
        >
          {loading ? "Signing in..." : "🚀 Sign In"}
        </button>

        <p className="text-center text-sm text-gray-600">
          Don’t have an account?{" "}
          <Link
            href="/signup"
            className="font-semibold text-pink-500 hover:underline"
          >
            Sign Up
          </Link>
        </p>
      </form>
    </div>
  );
}

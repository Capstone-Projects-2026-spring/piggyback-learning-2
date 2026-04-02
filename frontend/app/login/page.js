"use client";

import { useState, useContext, useEffect } from "react";
import { useRouter } from "next/navigation";
import { AuthContext } from "../context/AuthContext";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function LoginPage() {
  const router = useRouter();

  const { token, login } = useContext(AuthContext);
  useEffect(() => {
    if (token) router.replace("/");
  }, [token, router]);

  const [form, setForm] = useState({ username: "", password: "" });
  const [error, setError] = useState("");

  async function handleSubmit(e) {
    e.preventDefault();
    setError("");
    const res = await fetch(`${BASE_URL}/api/auth/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ ...form, role: "parent" }),
    });
    if (res.ok) {
      const data = await res.json();
      login(data.token, data.role);
      router.push("/");
    } else {
      const data = await res.json();
      setError(data.message || "Login failed.");
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-900 text-gray-100 p-4 sm:p-8">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-md space-y-6 rounded-lg bg-gray-800 p-6 sm:p-8 shadow-lg border border-gray-700"
      >
        <h1 className="text-center text-2xl sm:text-3xl font-bold text-indigo-400">
          Login
        </h1>

        {error && <p className="text-center text-sm text-red-500">{error}</p>}

        <input
          type="text"
          placeholder="Username"
          className="w-full rounded bg-gray-700 border border-gray-600 p-3 focus:ring-indigo-500 focus:border-indigo-500 text-gray-100"
          value={form.username}
          onChange={(e) => setForm({ ...form, username: e.target.value })}
          required
        />

        <input
          type="password"
          placeholder="Password"
          className="w-full rounded bg-gray-700 border border-gray-600 p-3 focus:ring-indigo-500 focus:border-indigo-500 text-gray-100"
          value={form.password}
          onChange={(e) => setForm({ ...form, password: e.target.value })}
          required
        />

        <button
          type="submit"
          className="w-full rounded bg-indigo-600 py-3 text-white hover:bg-indigo-700 transition"
        >
          Sign In
        </button>

        <p className="text-center text-sm text-gray-400">
          Don’t have an account?{" "}
          <a
            href="/signup"
            className="font-semibold text-indigo-300 hover:underline"
          >
            Sign Up
          </a>
        </p>
      </form>
    </div>
  );
}

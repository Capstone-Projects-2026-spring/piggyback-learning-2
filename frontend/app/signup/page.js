"use client";

import { useState, useContext, useEffect } from "react";
import { useRouter } from "next/navigation";
import { AuthContext } from "../context/AuthContxt";

const BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL;

export default function SignupPage() {
  const router = useRouter();
  const { token } = useContext(AuthContext);
  useEffect(() => {
    if (token) router.replace("/");
  }, [token, router]);

  const [form, setForm] = useState({
    name: "",
    username: "",
    password: "",
    parent_id: "",
  });
  const [error, setError] = useState("");

  async function handleSubmit(e) {
    e.preventDefault();
    setError("");

    const body = {
      ...form,
      role: "parent", // fixed to parent
      parent_id: null,
    };

    const res = await fetch(`${BASE_URL}/api/auth/signup`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });

    if (res.ok) router.push("/login");
    else {
      const data = await res.json();
      setError(data.message || "Signup failed.");
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-900 text-gray-100 p-4 sm:p-8">
      <form
        onSubmit={handleSubmit}
        className="w-full max-w-md space-y-6 rounded-lg bg-gray-800 p-6 sm:p-8 shadow-lg border border-gray-700"
      >
        <h1 className="text-center text-2xl sm:text-3xl font-bold text-indigo-300">
          Sign Up
        </h1>

        {error && <p className="text-center text-sm text-red-500">{error}</p>}

        <input
          type="text"
          placeholder="Name"
          className="w-full rounded bg-gray-700 border border-gray-600 p-3 focus:ring-indigo-500 focus:border-indigo-500 text-gray-100"
          value={form.name}
          onChange={(e) => setForm({ ...form, name: e.target.value })}
          required
        />

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
          Sign Up
        </button>

        <p className="text-center text-sm text-gray-400">
          Already have an account?{" "}
          <a
            href="/login"
            className="font-semibold text-indigo-300 hover:underline"
          >
            Login
          </a>
        </p>
      </form>
    </div>
  );
}

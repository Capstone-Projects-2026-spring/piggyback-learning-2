"use client";

import { useContext } from "react";
import Link from "next/link";
import { AuthContext } from "@/context/AuthContext";

export default function Navbar() {
  const { account, token, logout } = useContext(AuthContext);

  return (
    <nav className="flex items-center justify-between px-6 py-4 bg-linear-to-r from-pink-200 via-yellow-200 to-blue-200 shadow-md">
      {/* Logo */}
      <Link
        href="/"
        className="text-2xl font-extrabold text-pink-600 hover:scale-105 transition-transform"
      >
        🎒 Piggyback
      </Link>

      {/* Right Side */}
      <div className="flex items-center gap-4">
        {token ? (
          <>
            <span className="text-gray-700 font-medium">
              👋 Hi {account?.name}!
            </span>

            <button
              onClick={logout}
              className="px-4 py-2 rounded-xl bg-linear-to-r from-red-400 to-pink-400 text-white font-semibold hover:scale-105 transition transform shadow"
            >
              🚪 Logout
            </button>
          </>
        ) : (
          <>
            <Link
              href="/login"
              className="px-4 py-2 rounded-xl text-gray-700 font-semibold hover:bg-white/60 transition"
            >
              Login
            </Link>

            <Link
              href="/signup"
              className="px-4 py-2 rounded-xl bg-linear-to-r from-green-400 to-blue-400 text-white font-semibold hover:scale-105 transition transform shadow"
            >
              🎉 Sign Up
            </Link>
          </>
        )}
      </div>
    </nav>
  );
}

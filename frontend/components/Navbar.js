"use client";

import { useContext } from "react";
import Link from "next/link";
import { AuthContext } from "@/context/AuthContext";
import Lottie from "lottie-react";
import piggy from "../animations/piggy.json";

export default function Navbar() {
  const { account, token, logout } = useContext(AuthContext);

  return (
    <nav className="flex items-center justify-between px-6 py-4 bg-linear-to-r from-pink-200 via-yellow-200 to-blue-200 shadow-md">
      {/* Logo */}
      <Link
  href="/"
  className="flex items-center gap-2 text-2xl font-extrabold text-pink-600"
>
  <div className="w-12 h-12 flex items-center justify-center overflow-hidden">
    <Lottie
      animationData={piggy}
      loop
      style={{
        width: 70,
        height: 70,
        marginTop: -6,
      }}
    />
  </div>
  <span className="leading-none">Piggyback</span>
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

"use client";

import { useContext } from "react";
import Link from "next/link";
import { AuthContext } from "@/app/context/AuthContext";

export default function Navbar() {
  const { token, logout } = useContext(AuthContext);

  return (
    <nav className="flex items-center justify-between bg-gray-800 px-6 py-4 shadow-md">
      <Link href="/" className="text-xl font-bold text-indigo-300">
        Piggyback Learning
      </Link>

      <div className="flex gap-4">
        {token ? (
          <>
            <span className="text-gray-200">👤 Account</span>
            <button
              onClick={logout}
              className="bg-indigo-600 px-3 py-1 rounded hover:bg-indigo-700"
            >
              Logout
            </button>
          </>
        ) : (
          <>
            <Link href="/login" className="text-gray-200 hover:text-indigo-300">
              Login
            </Link>

            <Link
              href="/signup"
              className="bg-indigo-600 px-3 py-1 rounded text-white hover:bg-indigo-700"
            >
              Sign Up
            </Link>
          </>
        )}
      </div>
    </nav>
  );
}

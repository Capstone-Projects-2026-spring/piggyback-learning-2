"use client";

import { useContext, useEffect } from "react";
import { useRouter } from "next/navigation";
import { AuthContext } from "@/context/AuthContext";

export default function ProtectedRoute({ children }) {
  const router = useRouter();
  const { isLoggedIn } = useContext(AuthContext);

  useEffect(() => {
    if (!isLoggedIn) router.replace("/login");
  }, [isLoggedIn, router]);

  // Block rendering completely if not logged in
  if (isLoggedIn === false) return null;

  return children;
}

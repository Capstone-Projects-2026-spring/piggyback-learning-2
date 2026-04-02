"use client";

import { createContext, useState } from "react";

export const AuthContext = createContext();

export function AuthProvider({ children }) {
  const [token, setToken] = useState(
    typeof window !== "undefined" ? localStorage.getItem("token") : null,
  );
  const [role, setRole] = useState(
    typeof window !== "undefined" ? localStorage.getItem("role") : null,
  );
  const [account, setAccount] = useState(
    typeof window !== "undefined"
      ? JSON.parse(localStorage.getItem("account"))
      : null,
  );

  const login = (newToken, role, account) => {
    localStorage.setItem("token", newToken);
    localStorage.setItem("role", role);
    localStorage.setItem("account", JSON.stringify(account));
    setToken(newToken);
    setRole(role);
    setAccount(account);
  };

  const logout = () => {
    localStorage.removeItem("token");
    localStorage.removeItem("role");
    localStorage.removeItem("account");
    setToken(null);
    setRole(null);
    setAccount(null);
  };

  return (
    <AuthContext.Provider value={{ token, role, account, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

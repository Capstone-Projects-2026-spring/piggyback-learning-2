"use client";

import { createContext, useState, useEffect } from "react";

export const AuthContext = createContext();

export function AuthProvider({ children }) {
  const [token, setToken] = useState(null);
  const [role, setRole] = useState(null);
  const [account, setAccount] = useState(null);

  useEffect(() => {
    const storedToken = localStorage.getItem("token");
    const storedRole = localStorage.getItem("role");
    const storedAccount = localStorage.getItem("account");

    if (storedToken) setToken(storedToken);
    if (storedRole) setRole(storedRole);
    if (storedAccount) setAccount(JSON.parse(storedAccount));
  }, []);

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

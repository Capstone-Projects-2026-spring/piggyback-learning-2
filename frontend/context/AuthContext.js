"use client";

import { createContext, useState, useEffect } from "react";

export const AuthContext = createContext();

export function AuthProvider({ children }) {
  const [token, setToken] = useState(null);
  const [role, setRole] = useState(null);
  const [account, setAccount] = useState(null);
  const [parentUsername, setParentUsername] = useState(null);
  const [isLoggedIn, setIsLoggedIn] = useState(false);

  useEffect(() => {
    const storedToken = localStorage.getItem("token");
    const storedRole = localStorage.getItem("role");
    const storedAccount = localStorage.getItem("account");
    const parentUsername = localStorage.getItem("parentUsername");

    if (storedToken) setToken(storedToken);
    if (storedRole) setRole(storedRole);
    if (parentUsername) setParentUsername(parentUsername);
    if (storedAccount) setAccount(JSON.parse(storedAccount));
    if (
      storedToken &&
      storedRole &&
      storedAccount &&
      (storedRole === "kid" ? parentUsername : true)
    )
      setIsLoggedIn(true);
  }, []);

  const login = (newToken, role, account, parentUsername = null) => {
    localStorage.setItem("token", newToken);
    localStorage.setItem("role", role);
    localStorage.setItem("account", JSON.stringify(account));
    localStorage.setItem("parentUsername", parentUsername);
    setToken(newToken);
    setRole(role);
    setAccount(account);
    setParentUsername(parentUsername);
    setIsLoggedIn(true);
  };

  const logout = () => {
    localStorage.removeItem("token");
    localStorage.removeItem("role");
    localStorage.removeItem("account");
    localStorage.removeItem("parentUsername");
    setToken(null);
    setRole(null);
    setAccount(null);
    setParentUsername(null);
    setIsLoggedIn(false);
  };

  return (
    <AuthContext.Provider
      value={{
        isLoggedIn,
        token,
        role,
        account,
        parentUsername,
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

"use client";

import { createContext, useContext, useState } from "react";

const PiggyContext = createContext(null);

export function PiggyProvider({ children }) {
  const [piggyMode, setPiggyMode] = useState("default");
  const [piggyText, setPiggyText] = useState("Hi! Let’s get started 🚀");

  return (
    <PiggyContext.Provider
      value={{ piggyMode, setPiggyMode, piggyText, setPiggyText }}
    >
      {children}
    </PiggyContext.Provider>
  );
}

export function usePiggy() {
  const ctx = useContext(PiggyContext);
  if (!ctx) throw new Error("usePiggy must be used inside PiggyProvider");
  return ctx;
}
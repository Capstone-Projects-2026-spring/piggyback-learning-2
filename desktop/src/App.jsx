import { useEffect, useState } from "react";
import PeppaOrb from "./components/PeppaOrb.jsx";
import EnrollmentOverlay from "./components/EnrollmentOverlay.jsx";
import { startPeppa, commandBus } from "./lib/stt/index.js";

export default function App() {
  const [mode, setMode] = useState("loading");

  useEffect(() => {
    // Register BEFORE startPeppa so we never miss an event
    const off = commandBus.onEnrollment((data) => {
      console.log("[App] enrollment stage:", data.stage, data);
      if (data.stage === "done") {
        setMode("ready");
      } else {
        setMode("enrolling");
      }
    });

    // Give the listener a tick to register, then start
    setTimeout(() => {
      startPeppa().catch(console.error);
    }, 100);

    // Fall back to ready after 6s if no enrollment event ever arrives
    const fallback = setTimeout(() => {
      setMode((m) => (m === "loading" ? "ready" : m));
    }, 6000);

    return () => {
      off();
      clearTimeout(fallback);
    };
  }, []);

  if (mode === "loading") {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen bg-gradient-to-b from-pink-50 to-white select-none">
        <p className="text-sm text-pink-300 animate-pulse">Starting Peppa…</p>
      </div>
    );
  }

  if (mode === "enrolling") {
    return <EnrollmentOverlay onDone={() => setMode("ready")} />;
  }

  return <PeppaOrb />;
}

import { useEffect, useState } from "react";
import PeppaOrb from "./components/PeppaOrb.jsx";
import EnrollmentOverlay from "./components/EnrollmentOverlay.jsx";
import VideoPanel from "./components/VideoPanel.jsx";
import { commandBus } from "./lib/stt/index.js";

export default function App() {
  const [mode, setMode] = useState("loading");
  const [kidEnrolling, setKidEnrolling] = useState(false);
  const [showVideos, setShowVideos] = useState(false);

  useEffect(() => {
    const offEnrollment = commandBus.onEnrollment((data) => {
      console.log("[App] enrollment event:", data.stage, "flow:", data.flow);
      if (data.flow === "parent") {
        if (data.stage === "done") {
          setMode("ready");
        } else {
          setMode("enrolling");
        }
      }
      if (data.flow === "kid") {
        if (data.stage === "kid_done") {
          setTimeout(() => setKidEnrolling(false), 3000);
        } else {
          setKidEnrolling(true);
        }
      }
    });

    const offVideos = commandBus.on("my_videos", () => setShowVideos(true));
    const offSearch = commandBus.on("search", () => setShowVideos(true));
    const offRecs = commandBus.on("recommendations", () => setShowVideos(true));

    const fallback = setTimeout(() => {
      setMode((m) => (m === "loading" ? "ready" : m));
    }, 6000);

    return () => {
      offEnrollment();
      offVideos();
      offSearch();
      offRecs();
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
    return <EnrollmentOverlay flow="parent" onDone={() => setMode("ready")} />;
  }

  return (
    <>
      <PeppaOrb />
      {showVideos && <VideoPanel onClose={() => setShowVideos(false)} />}
      {kidEnrolling && (
        <div className="fixed inset-0 z-50 bg-black/40 backdrop-blur-sm">
          <EnrollmentOverlay flow="kid" onDone={() => setKidEnrolling(false)} />
        </div>
      )}
    </>
  );
}

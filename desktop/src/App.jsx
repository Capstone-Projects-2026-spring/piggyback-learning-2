import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import PeppaOrb from "./components/PeppaOrb.jsx";
import EnrollmentOverlay from "./components/EnrollmentOverlay.jsx";
import VideoPanel from "./components/VideoPanel.jsx";
import { commandBus } from "./lib/stt/index.js";

export default function App() {
  const [mode, setMode] = useState("loading");
  const [role, setRole] = useState(null);
  const [kidEnrolling, setKidEnrolling] = useState(false);
  const [showVideos, setShowVideos] = useState(false);
  const [myVideosData, setMyVideosData] = useState(null); // pre-fetched before panel opens

  useEffect(() => {
    const offEnrollment = commandBus.onEnrollment((data) => {
      if (data.flow === "parent") {
        if (data.stage === "done") {
          setMode("ready");
          setRole("parent");
        } else setMode("enrolling");
      }
      if (data.flow === "kid") {
        if (data.stage === "kid_done")
          setTimeout(() => setKidEnrolling(false), 3000);
        else setKidEnrolling(true);
      }
    });

    const offSearch = commandBus.on("search", () => {
      setMyVideosData(null);
      setShowVideos(true);
    });

    const offRecs = commandBus.on("recommendations", () => {
      setMyVideosData(null);
      setShowVideos(true);
    });

    // Listen for my-videos BEFORE opening panel — store data first, then open
    let unlistenMyVideos;
    listen("peppa://my-videos", ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      setMyVideosData(data); // store it
      setShowVideos(true); // then open panel — data is already in state
    }).then((fn) => {
      unlistenMyVideos = fn;
    });

    let unlistenRecs;
    listen("peppa://recommendations", () => {
      setShowVideos(true);
    }).then((fn) => {
      unlistenRecs = fn;
    });

    const fallback = setTimeout(() => {
      setMode((m) => (m === "loading" ? "ready" : m));
    }, 6000);

    return () => {
      offEnrollment();
      offSearch();
      offRecs();
      unlistenMyVideos?.();
      unlistenRecs?.();
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
    return (
      <EnrollmentOverlay
        flow="parent"
        onDone={() => {
          setMode("ready");
          setRole("parent");
        }}
      />
    );
  }

  return (
    <>
      <PeppaOrb />
      {showVideos && (
        <VideoPanel
          role={role}
          initialMyVideos={myVideosData}
          onClose={() => {
            setShowVideos(false);
            setMyVideosData(null);
          }}
        />
      )}
      {kidEnrolling && (
        <div className="fixed inset-0 z-50 bg-black/40 backdrop-blur-sm">
          <EnrollmentOverlay flow="kid" onDone={() => setKidEnrolling(false)} />
        </div>
      )}
    </>
  );
}

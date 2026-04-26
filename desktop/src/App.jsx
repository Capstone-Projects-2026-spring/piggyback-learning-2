import { useEffect, useState } from "react";
import { commandBus } from "@/lib";
import { useTauriListener } from "@/hooks";
import Orb from "@/components/orb/Orb.jsx";
import EnrollmentOverlay from "@/components/enrollment/EnrollmentOverlay.jsx";
import VideoPanel from "@/components/video/VideoPanel.jsx";
import { speak } from "@/utils";

const LOADING_FALLBACK_MS = 6000;

export default function App() {
  const [mode, setMode] = useState("loading");
  const [role, setRole] = useState(null);
  const [kidEnrolling, setKidEnrolling] = useState(false);
  const [showVideos, setShowVideos] = useState(false);

  useTauriListener("orb://my-videos", () => {
    setShowVideos(true);
  });

  useTauriListener("orb://recommendations", () => {
    setShowVideos(true);
  });

  useEffect(() => {
    const offEnrollment = commandBus.onEnrollment((data) => {
      if (data.flow === "parent") {
        setMode(data.stage === "done" ? "ready" : "enrolling");
        if (data.stage === "done") setRole("parent");
      }
      if (data.flow === "kid") {
        if (data.stage === "done") {
          setTimeout(() => setKidEnrolling(false), 3000);
        } else {
          setKidEnrolling(true);
        }
      }
    });

    const offSearch = commandBus.on("search", () => setShowVideos(true));

    // Fallback in case Rust never fires an enrollment event (already enrolled).
    const fallback = setTimeout(() => {
      setMode((m) => {
        if (m === "loading") {
          speak("Hey! I'm Jarvis. Say my name to get started.");
          return "ready";
        }
        return m;
      });
    }, LOADING_FALLBACK_MS);

    return () => {
      offEnrollment();
      offSearch();
      clearTimeout(fallback);
    };
  }, []);

  if (mode === "loading") {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen bg-gradient-to-b from-pink-50 to-white select-none">
        <p className="text-sm text-pink-300 animate-pulse">Starting up…</p>
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
      <Orb />

      {showVideos && (
        <VideoPanel role={role} onClose={() => setShowVideos(false)} />
      )}

      {kidEnrolling && (
        <div className="fixed inset-0 z-50 bg-black/40 backdrop-blur-sm">
          <EnrollmentOverlay flow="kid" onDone={() => setKidEnrolling(false)} />
        </div>
      )}
    </>
  );
}

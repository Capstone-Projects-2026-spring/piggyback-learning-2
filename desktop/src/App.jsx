import { useEffect, useState, lazy, Suspense } from "react";
import { invoke } from "@tauri-apps/api/core";
import { startOrb, stopOrb, commandBus } from "@/lib";
import { useTauriListener } from "@/hooks";
import { speak } from "@/utils";
import Orb from "@/components/orb/Orb.jsx";
import EnrollmentOverlay from "@/components/enrollment/EnrollmentOverlay.jsx";

const VideoPanel = lazy(() => import("@/components/video/VideoPanel.jsx"));
const ResultsPanel = lazy(
  () => import("@/components/results/ResultsPanel.jsx"),
);

export default function App() {
  const [mode, setMode] = useState("loading");
  const [role, setRole] = useState(null);
  const [kidEnrolling, setKidEnrolling] = useState(false);
  const [showVideos, setShowVideos] = useState(false);
  const [resultsData, setResultsData] = useState(null);
  const [enrollmentEvent, setEnrollmentEvent] = useState(null);
  const [kidEnrollmentEvent, setKidEnrollmentEvent] = useState(null);

  useEffect(() => {
    // Start listeners immediately so enrollment events are never missed.
    startOrb();

    const offEnrollment = commandBus.onEnrollment((data) => {
      if (data.flow === "parent") {
        setEnrollmentEvent(data);
        if (data.stage === "done") {
          setMode("ready");
          setRole("parent");
        } else {
          setMode("enrolling");
        }
      }
      if (data.flow === "kid") {
        setKidEnrollmentEvent(data);
        if (data.stage === "kid_done") {
          setTimeout(() => setKidEnrolling(false), 3000);
        } else {
          setKidEnrolling(true);
        }
      }
    });

    const offSearch = commandBus.on("search", () => setShowVideos(true));
    const offMyVideos = commandBus.on("my_videos", () => setShowVideos(true));

    const init = async () => {
      // Poll until backend is fully initialised.
      while (!(await invoke("is_backend_ready").catch(() => false))) {
        await new Promise((r) => setTimeout(r, 100));
      }

      // Handshake - backend starts onboarding if needed and returns whether
      // onboarding is required so we know what to show.
      const needsOnboarding = await invoke("frontend_ready").catch(() => false);
      console.log("[app] frontend_ready — needsOnboarding:", needsOnboarding);

      if (!needsOnboarding) {
        speak("Hey! I'm Jarvis. Say my name to get started.");
        setMode("ready");
      }
      // If onboarding needed, stay in "loading" until the greet enrollment
      // event arrives and the onEnrollment handler sets mode to "enrolling".
    };

    init();

    return () => {
      stopOrb();
      offEnrollment();
      offSearch();
      offMyVideos();
    };
  }, []);

  useTauriListener("orb://my-videos", () => setShowVideos(true));
  useTauriListener("orb://recommendations", () => setShowVideos(true));
  useTauriListener("orb://answers", (data) => {
    const answers = Array.isArray(data) ? data : (data.answers ?? []);
    setResultsData(answers);
  });

  if (mode === "loading") {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen bg-linear-to-b from-pink-50 to-white select-none">
        <p className="text-sm text-pink-300 animate-pulse">Starting up…</p>
      </div>
    );
  }

  if (mode === "enrolling") {
    return (
      <EnrollmentOverlay
        flow="parent"
        currentEvent={enrollmentEvent}
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
        <Suspense fallback={null}>
          <VideoPanel role={role} onClose={() => setShowVideos(false)} />
        </Suspense>
      )}

      {resultsData && (
        <Suspense fallback={null}>
          <ResultsPanel
            answers={resultsData}
            onClose={() => setResultsData(null)}
          />
        </Suspense>
      )}

      {kidEnrolling && (
        <div className="fixed inset-0 z-50 bg-black/40 backdrop-blur-sm">
          <EnrollmentOverlay
            flow="kid"
            currentEvent={kidEnrollmentEvent}
            onDone={() => setKidEnrolling(false)}
          />
        </div>
      )}
    </>
  );
}

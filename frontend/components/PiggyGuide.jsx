"use client";

import Image from "next/image";
import { usePathname } from "next/navigation";
import { usePiggy } from "@/context/PiggyContext";

import piggy from "../animations/piggy.png";
import piggyTalk from "../animations/piggy_talk.png";
import piggyWatch from "../animations/piggy_watch.png";
import piggykids from "../animations/piggy_kids.png";
import piggydownload from "../animations/piggy_download.png";
import piggycreate from "../animations/piggy_newkid.png";
import piggylock from "../animations/piggy_lock.png";
import piggytag from "../animations/piggy_tag.png";
import piggyrecommend from "../animations/piggy_recommend.png";
import piggyassign from "../animations/piggy_assign.png";

export default function PiggyGuide() {
  const pathname = usePathname();
  const { piggyMode, piggyText } = usePiggy();

  let pig = piggy;
  let text = piggyText || "Hi! Let’s get started 🐷";

  if (pathname === "/" && piggyMode === "talk") {
  pig = piggycreate;
}

  if (pathname === "/login" || pathname === "/signup") {
  pig = piggyMode === "watch" ? piggylock : piggy;
  text = piggyText || (pathname === "/login"
    ? "Welcome back! Sign in 😊"
    : "Let’s create your account!");
}

  if (pathname === "/watch") {
    pig = piggyWatch;
    text = "Focus! Let’s learn 📚";
  }
  
  const pigMap = {
  tags: piggytag,
  recommended: piggyrecommend,
  search: piggykids,
  assigned: piggyassign,
};

if (pathname.startsWith("/kids/")) {
  pig = pigMap[piggyMode] || piggytag;
  text = piggyText || "Pick a video for your kid to learn!";
}
const stepMap = {
  "download": "Downloading your video... ⬇️",
  "frames": "Extracting frames... 📸",
  "tags": "Generating tags... 🏷️",
  "questions": "Creating questions... 🧠",
};

if (pathname.includes("/process")) {
  pig = piggydownload;
  text = piggyText || "Processing...";
  
}
if (pathname.includes("/watch")) return null;

  return (
    <div className="fixed bottom-25 right-6 z-[9999] flex flex-col items-center">
      
      {/* Speech bubble */}
       <div className="relative mb-2 bg-white px-4 py-2 rounded-xl shadow-md text-sm text-gray-800 font-medium text-center">
    {text}
    <div className="absolute bottom-[-6px] left-1/2 -translate-x-1/2 w-3 h-3 bg-white rotate-45"></div>
  </div>

      {/* Pig image */}
      <Image
        src={pig}
        alt="Piggy"
        width={120}
        height={120}
        className="object-contain animate-[float_3s_ease-in-out_infinite]"
      />
    </div>
  );
}
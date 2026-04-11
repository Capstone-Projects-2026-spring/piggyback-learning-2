"use client";

import Lottie from "lottie-react";
import piggy from "../animations/piggy.json";

export default function PiggyComp() {
  return (
    <div
      style={{
        position: "fixed",
        top: 90,
        left: 10,
        width: 120,
        zIndex: 9999,
      }}
    >
      <Lottie animationData={piggy} loop />
    </div>
  );
}
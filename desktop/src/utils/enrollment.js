export function normaliseStage(stage) {
  return stage.replace(/^kid_/, "");
}

export const THEME = {
  parent: {
    bg: "from-pink-50 to-white",
    pill: "bg-pink-100 border-pink-200 text-pink-500",
    orb: {
      greet: "ring-pink-100",
      name_confirmed: "ring-pink-200",
      prompt: "ring-blue-200",
      done: "ring-green-300",
    },
    dot: "bg-pink-400",
  },
  kid: {
    bg: "from-violet-50 to-white",
    pill: "bg-violet-100 border-violet-200 text-violet-500",
    orb: {
      greet: "ring-violet-100",
      name_confirmed: "ring-violet-200",
      prompt: "ring-blue-200",
      done: "ring-green-300",
    },
    dot: "bg-violet-400",
  },
};

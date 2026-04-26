export const QTYPE_LABELS = {
  character: "Character",
  setting: "Setting",
  feeling: "Feeling",
  action: "Action",
  causal: "Cause & Effect",
  outcome: "Outcome",
  prediction: "Prediction",
};

export const QTYPE_COLORS = {
  character: "bg-pink-50 border-pink-200 text-pink-600",
  setting: "bg-blue-50 border-blue-200 text-blue-600",
  feeling: "bg-yellow-50 border-yellow-200 text-yellow-600",
  action: "bg-green-50 border-green-200 text-green-600",
  causal: "bg-violet-50 border-violet-200 text-violet-600",
  outcome: "bg-orange-50 border-orange-200 text-orange-600",
  prediction: "bg-teal-50 border-teal-200 text-teal-600",
};

export const formatTime = (s) =>
  `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;

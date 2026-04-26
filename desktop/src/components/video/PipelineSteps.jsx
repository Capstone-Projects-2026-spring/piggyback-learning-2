import Checkmark from "../ui/Checkmark.jsx";
import { PIPELINE_STEPS } from "@/utils";

export default function PipelineSteps({ stage, hasQuestions }) {
  const stageOrder = PIPELINE_STEPS.map((s) => s.key);
  const currentIdx = stage ? stageOrder.indexOf(stage) : -1;

  return (
    <div className="flex flex-col gap-1.5">
      {PIPELINE_STEPS.map((step, i) => {
        const done = hasQuestions || currentIdx > i;
        const active = currentIdx === i;
        return (
          <div key={step.key} className="flex items-center gap-2">
            <div
              className={`w-4 h-4 rounded-full flex items-center justify-center flex-shrink-0 ${
                done
                  ? "bg-green-400"
                  : active
                    ? "bg-amber-400 animate-pulse"
                    : "bg-gray-100"
              }`}
            >
              {done ? (
                <Checkmark className="w-2.5 h-2.5 text-white" />
              ) : (
                <span className="w-1.5 h-1.5 rounded-full bg-gray-300" />
              )}
            </div>
            <span
              className={`text-xs ${done ? "text-green-600" : active ? "text-amber-500 font-medium" : "text-gray-300"}`}
            >
              {step.label}
            </span>
          </div>
        );
      })}
    </div>
  );
}

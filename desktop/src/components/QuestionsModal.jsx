import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const QTYPE_LABELS = {
  character: "Character",
  setting: "Setting",
  feeling: "Feeling",
  action: "Action",
  causal: "Cause & Effect",
  outcome: "Outcome",
  prediction: "Prediction",
};

const QTYPE_COLORS = {
  character: "bg-pink-50 border-pink-200 text-pink-600",
  setting: "bg-blue-50 border-blue-200 text-blue-600",
  feeling: "bg-yellow-50 border-yellow-200 text-yellow-600",
  action: "bg-green-50 border-green-200 text-green-600",
  causal: "bg-violet-50 border-violet-200 text-violet-600",
  outcome: "bg-orange-50 border-orange-200 text-orange-600",
  prediction: "bg-teal-50 border-teal-200 text-teal-600",
};

function EditableField({ value, onChange, multiline = false, className = "" }) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);

  const commit = () => {
    setEditing(false);
    if (draft.trim() !== value) onChange(draft.trim());
  };

  if (editing) {
    return multiline ? (
      <textarea
        autoFocus
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        rows={2}
        className={`w-full text-sm rounded-lg border border-pink-300 px-2 py-1 resize-none focus:outline-none focus:ring-1 focus:ring-pink-300 ${className}`}
      />
    ) : (
      <input
        autoFocus
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        className={`w-full text-sm rounded-lg border border-pink-300 px-2 py-1 focus:outline-none focus:ring-1 focus:ring-pink-300 ${className}`}
      />
    );
  }

  return (
    <span
      onClick={() => {
        setDraft(value);
        setEditing(true);
      }}
      className={`cursor-text hover:bg-pink-50 rounded px-0.5 transition-colors ${className}`}
      title="Click to edit"
    >
      {value}
    </span>
  );
}

export default function QuestionsModal({ videoId, segments, onClose }) {
  const [segmentIndex, setSegmentIndex] = useState(0);
  const [bestQuestions, setBestQuestions] = useState(() =>
    Object.fromEntries(
      segments.map((s) => [s.segment.id, s.segment.best_question]),
    ),
  );
  const [expandedQuestion, setExpandedQuestion] = useState(null);
  const [editedSegments, setEditedSegments] = useState(() =>
    segments.map((seg) => ({
      ...seg,
      questions: seg.questions.map((q) => ({ ...q })),
    })),
  );
  const [saving, setSaving] = useState(false);
  const [savedOk, setSavedOk] = useState(false);

  const segment = editedSegments[segmentIndex];
  if (!segment) return null;

  const { id: segmentId } = segment.segment;
  const questions = [...(segment.questions ?? [])].sort(
    (a, b) => (a.rank ?? 99) - (b.rank ?? 99),
  );
  const currentBest = bestQuestions[segmentId];

  const formatTime = (s) =>
    `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;

  const setBest = (question) =>
    setBestQuestions((prev) => ({ ...prev, [segmentId]: question }));

  const updateQuestion = (qtype, field, value) => {
    setEditedSegments((prev) =>
      prev.map((seg, i) =>
        i !== segmentIndex
          ? seg
          : {
              ...seg,
              questions: seg.questions.map((q) =>
                q.qtype === qtype ? { ...q, [field]: value } : q,
              ),
            },
      ),
    );
    // Keep best banner in sync if the question text being edited is the current best
    if (field === "question") {
      setBestQuestions((prev) => {
        const old = segment.questions.find((q) => q.qtype === qtype)?.question;
        if (prev[segmentId] === old) return { ...prev, [segmentId]: value };
        return prev;
      });
    }
  };

  const handleSave = async () => {
    setSaving(true);
    setSavedOk(false);

    const updates = editedSegments.map((seg) => ({
      segment_id: seg.segment.id,
      best_question: bestQuestions[seg.segment.id] ?? seg.segment.best_question,
      questions: seg.questions.map((q) => ({
        segment_id: seg.segment.id,
        qtype: q.qtype,
        question: q.question,
        answer: q.answer,
        followup_correct_question: q.followup_correct_question ?? null,
        followup_correct_answer: q.followup_correct_answer ?? null,
        followup_wrong_question: q.followup_wrong_question ?? null,
        followup_wrong_answer: q.followup_wrong_answer ?? null,
      })),
    }));

    try {
      await invoke("save_questions", { updates });
      setSavedOk(true);
      setTimeout(() => setSavedOk(false), 2000);
    } catch (e) {
      console.error("[QuestionsModal] save failed:", e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative w-full max-w-lg bg-white rounded-3xl shadow-2xl flex flex-col max-h-[88vh]">
        {/* Header */}
        <div className="flex items-center justify-between px-5 pt-5 pb-3 border-b border-gray-100 flex-shrink-0">
          <div>
            <h2 className="text-base font-bold text-gray-800">Questions</h2>
            <p className="text-xs text-gray-400 mt-0.5">
              Tap any text to edit · ⭐ to set best
            </p>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 rounded-full bg-gray-100 flex items-center justify-center text-gray-400 hover:text-gray-600 transition-colors"
          >
            <svg
              className="w-4 h-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* Segment tabs */}
        {editedSegments.length > 1 && (
          <div className="flex gap-2 px-5 py-3 overflow-x-auto flex-shrink-0 border-b border-gray-50">
            {editedSegments.map((seg, i) => (
              <button
                key={seg.segment.id}
                onClick={() => {
                  setSegmentIndex(i);
                  setExpandedQuestion(null);
                }}
                className={`flex-shrink-0 text-xs px-3 py-1.5 rounded-full border transition-colors ${
                  i === segmentIndex
                    ? "bg-pink-400 border-pink-400 text-white"
                    : "bg-white border-gray-200 text-gray-500 hover:border-pink-200"
                }`}
              >
                {formatTime(seg.segment.start_seconds)}–
                {formatTime(seg.segment.end_seconds)}
              </button>
            ))}
          </div>
        )}

        {/* Best question banner */}
        {currentBest && (
          <div className="mx-5 mt-3 mb-1 px-4 py-3 rounded-2xl bg-pink-50 border border-pink-200 flex-shrink-0">
            <p className="text-xs text-pink-400 font-medium mb-1">
              ⭐ Best Question
            </p>
            <p className="text-sm text-pink-700 font-medium leading-snug">
              {currentBest}
            </p>
          </div>
        )}

        {/* Scrollable questions */}
        <div className="flex-1 overflow-y-auto px-5 py-3 flex flex-col gap-3">
          {questions.map((q) => {
            const isExpanded = expandedQuestion === q.qtype;
            const isBest = currentBest === q.question;

            return (
              <div
                key={q.qtype}
                className={`rounded-2xl border transition-all duration-200 ${
                  isBest ? "border-pink-300 shadow-sm" : "border-gray-100"
                }`}
              >
                {/* Question row */}
                <div className="flex items-start gap-3 p-4">
                  <span
                    className={`flex-shrink-0 text-xs px-2 py-0.5 rounded-full border font-medium ${
                      QTYPE_COLORS[q.qtype] ??
                      "bg-gray-50 border-gray-200 text-gray-500"
                    }`}
                  >
                    {QTYPE_LABELS[q.qtype] ?? q.qtype}
                  </span>

                  <div className="flex-1 min-w-0 flex flex-col gap-1">
                    <EditableField
                      value={q.question}
                      multiline
                      onChange={(v) => updateQuestion(q.qtype, "question", v)}
                      className="text-gray-800 font-medium leading-snug"
                    />
                    <div className="flex items-center gap-1 text-xs text-gray-400">
                      <span>Answer:</span>
                      <EditableField
                        value={q.answer}
                        onChange={(v) => updateQuestion(q.qtype, "answer", v)}
                        className="font-medium text-gray-600"
                      />
                    </div>
                  </div>

                  <div className="flex flex-col items-end gap-2 flex-shrink-0">
                    <button
                      onClick={() => setBest(q.question)}
                      title={isBest ? "Current best" : "Set as best"}
                      className={`text-base transition-opacity ${
                        isBest ? "opacity-100" : "opacity-30 hover:opacity-70"
                      }`}
                    >
                      ⭐
                    </button>
                    <button
                      onClick={() =>
                        setExpandedQuestion(isExpanded ? null : q.qtype)
                      }
                      className="text-gray-300 hover:text-gray-400 transition-colors"
                    >
                      <svg
                        className={`w-4 h-4 transition-transform duration-200 ${
                          isExpanded ? "rotate-180" : ""
                        }`}
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M19 9l-7 7-7-7"
                        />
                      </svg>
                    </button>
                  </div>
                </div>

                {/* Follow-ups */}
                {isExpanded && (
                  <div className="px-4 pb-4 flex flex-col gap-3 border-t border-gray-50">
                    <p className="text-xs text-gray-400 font-medium uppercase tracking-wide mt-3">
                      Follow-up Questions
                    </p>

                    <div className="rounded-xl bg-green-50 border border-green-100 px-3 py-2.5 flex flex-col gap-1.5">
                      <p className="text-xs text-green-500 font-medium">
                        ✓ If correct
                      </p>
                      <EditableField
                        value={q.followup_correct_question ?? ""}
                        multiline
                        onChange={(v) =>
                          updateQuestion(
                            q.qtype,
                            "followup_correct_question",
                            v,
                          )
                        }
                        className="text-green-800 text-sm leading-snug"
                      />
                      <div className="flex items-center gap-1 text-xs text-green-500">
                        <span>Answer:</span>
                        <EditableField
                          value={q.followup_correct_answer ?? ""}
                          onChange={(v) =>
                            updateQuestion(
                              q.qtype,
                              "followup_correct_answer",
                              v,
                            )
                          }
                          className="font-medium text-green-700"
                        />
                      </div>
                    </div>

                    <div className="rounded-xl bg-red-50 border border-red-100 px-3 py-2.5 flex flex-col gap-1.5">
                      <p className="text-xs text-red-400 font-medium">
                        ✗ If wrong
                      </p>
                      <EditableField
                        value={q.followup_wrong_question ?? ""}
                        multiline
                        onChange={(v) =>
                          updateQuestion(q.qtype, "followup_wrong_question", v)
                        }
                        className="text-red-800 text-sm leading-snug"
                      />
                      <div className="flex items-center gap-1 text-xs text-red-400">
                        <span>Answer:</span>
                        <EditableField
                          value={q.followup_wrong_answer ?? ""}
                          onChange={(v) =>
                            updateQuestion(q.qtype, "followup_wrong_answer", v)
                          }
                          className="font-medium text-red-700"
                        />
                      </div>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
          <div className="h-2" />
        </div>

        {/* Save button — fixed at bottom */}
        <div className="px-5 py-4 border-t border-gray-100 flex-shrink-0">
          <button
            onClick={handleSave}
            disabled={saving}
            className={`w-full py-2.5 rounded-2xl text-sm font-semibold transition-all duration-200 ${
              savedOk
                ? "bg-green-500 text-white"
                : saving
                  ? "bg-pink-200 text-white cursor-not-allowed"
                  : "bg-pink-400 text-white hover:bg-pink-500 active:scale-95"
            }`}
          >
            {savedOk ? "✓ Saved" : saving ? "Saving…" : "Save Changes"}
          </button>
        </div>
      </div>
    </div>
  );
}

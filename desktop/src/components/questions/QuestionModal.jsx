import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import CloseButton from "../ui/CloseButton.jsx";
import Backdrop from "../ui/Backdrop.jsx";
import SegmentTabs from "./SegmentTabs.jsx";
import QuestionCard from "./QuestionCard.jsx";

export default function QuestionModal({ videoId, segments, onClose }) {
  const [segmentIndex, setSegmentIndex] = useState(0);
  const [expandedQuestion, setExpandedQuestion] = useState(null);
  const [bestQuestions, setBestQuestions] = useState(() =>
    Object.fromEntries(
      segments.map((s) => [s.segment.id, s.segment.best_question]),
    ),
  );
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
    if (field === "question") {
      setBestQuestions((prev) => {
        const old = segment.questions.find((q) => q.qtype === qtype)?.question;
        return prev[segmentId] === old ? { ...prev, [segmentId]: value } : prev;
      });
    }
  };

  const handleSave = async () => {
    setSaving(true);
    setSavedOk(false);
    try {
      const updates = editedSegments.map((seg) => ({
        segment_id: seg.segment.id,
        best_question:
          bestQuestions[seg.segment.id] ?? seg.segment.best_question,
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
      <Backdrop onClick={onClose} />
      <div className="relative w-full max-w-lg bg-white rounded-3xl shadow-2xl flex flex-col max-h-[88vh]">
        <div className="flex items-center justify-between px-5 pt-5 pb-3 border-b border-gray-100 flex-shrink-0">
          <div>
            <h2 className="text-base font-bold text-gray-800">Questions</h2>
            <p className="text-xs text-gray-400 mt-0.5">
              Tap any text to edit · ⭐ to set best
            </p>
          </div>
          <CloseButton
            onClick={onClose}
            className="w-8 h-8 bg-gray-100 text-gray-400 hover:text-gray-600"
          />
        </div>

        <SegmentTabs
          segments={editedSegments}
          activeIndex={segmentIndex}
          onSelect={(i) => {
            setSegmentIndex(i);
            setExpandedQuestion(null);
          }}
        />

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

        <div className="flex-1 overflow-y-auto px-5 py-3 flex flex-col gap-3">
          {questions.map((q) => (
            <QuestionCard
              key={q.qtype}
              q={q}
              isBest={currentBest === q.question}
              isExpanded={expandedQuestion === q.qtype}
              onSetBest={() => setBest(q.question)}
              onToggleExpand={() =>
                setExpandedQuestion(
                  expandedQuestion === q.qtype ? null : q.qtype,
                )
              }
              onUpdate={(field, value) => updateQuestion(q.qtype, field, value)}
            />
          ))}
          <div className="h-2" />
        </div>

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

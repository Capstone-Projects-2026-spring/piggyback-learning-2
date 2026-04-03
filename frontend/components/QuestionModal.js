import { useState } from "react";

export default function QuestionModal({ question, onClose }) {
  const [answer, setAnswer] = useState("");

  if (!question) return null;

  const handleSubmit = () => {
    onClose(answer);
    setAnswer("");
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white p-6 rounded-2xl shadow-lg max-w-md w-full text-center animate-fadeIn">
        <h2 className="text-2xl font-bold mb-4 text-purple-700">
          Question Time!
        </h2>
        <p className="text-lg mb-4 text-gray-800">{question}</p>
        <input
          type="text"
          value={answer}
          onChange={(e) => setAnswer(e.target.value)}
          placeholder="Type your answer..."
          className="w-full px-4 py-2 mb-4 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500 text-gray-800"
        />
        <button
          onClick={handleSubmit}
          className="bg-purple-500 text-white px-6 py-2 rounded-full hover:bg-purple-600 transition"
        >
          Submit
        </button>
      </div>
    </div>
  );
}

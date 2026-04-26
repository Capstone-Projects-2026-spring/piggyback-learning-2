import { useState } from "react";

export default function EditableField({
  value,
  onChange,
  multiline = false,
  className = "",
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);

  const commit = () => {
    setEditing(false);
    if (draft.trim() !== value) onChange(draft.trim());
  };

  if (editing) {
    const shared = {
      autoFocus: true,
      value: draft,
      onChange: (e) => setDraft(e.target.value),
      onBlur: commit,
      className: `w-full text-sm rounded-lg border border-pink-300 px-2 py-1 focus:outline-none focus:ring-1 focus:ring-pink-300 ${className}`,
    };
    return multiline ? (
      <textarea
        {...shared}
        rows={2}
        className={`${shared.className} resize-none`}
      />
    ) : (
      <input {...shared} />
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

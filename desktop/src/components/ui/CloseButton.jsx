export default function CloseButton({ onClick, className = "" }) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center justify-center rounded-full transition-colors ${className}`}
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
  );
}

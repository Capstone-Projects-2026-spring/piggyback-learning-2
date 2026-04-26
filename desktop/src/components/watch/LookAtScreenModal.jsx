export default function LookAtScreenModal() {
  return (
    <div className="absolute inset-0 bg-black/80 backdrop-blur-sm flex flex-col items-center justify-center gap-4 z-20">
      <div className="text-5xl animate-bounce">👀</div>
      <p className="text-white text-lg font-semibold">
        Hey, look at the screen!
      </p>
      <p className="text-white/50 text-sm">
        The video will resume when you're back
      </p>
    </div>
  );
}

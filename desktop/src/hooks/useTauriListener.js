import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

export function useTauriListener(event, handler) {
  const handlerRef = useRef(handler);
  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    let cancelled = false;
    let unlisten = null;
    listen(event, ({ payload }) => {
      const data = typeof payload === "string" ? JSON.parse(payload) : payload;
      handlerRef.current(data);
    }).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [event]);
}

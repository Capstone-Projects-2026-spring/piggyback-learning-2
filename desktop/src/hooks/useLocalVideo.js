import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const CHUNK_SIZE = 1024 * 1024 * 2; // 2MB chunks

export function useLocalVideo(videoPath) {
  const [blobUrl, setBlobUrl] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    if (!videoPath) return;

    let cancelled = false;
    let url = null;

    async function load() {
      setLoading(true);
      setError(null);

      try {
        console.log("[useLocalVideo] starting load for:", videoPath);
        const fileSize = await invoke("get_video_file_size", {
          path: videoPath,
        });
        console.log("[useLocalVideo] file size:", fileSize);
        const chunks = [];
        let offset = 0;

        while (offset < fileSize) {
          if (cancelled) return;
          const length = Math.min(CHUNK_SIZE, fileSize - offset);
          const chunk = await invoke("read_video_chunk", {
            path: videoPath,
            offset,
            length,
          });
          chunks.push(new Uint8Array(chunk));
          offset += length;
        }

        if (cancelled) return;

        const blob = new Blob(chunks, { type: "video/mp4" });
        console.log("[useLocalVideo] blob created, size:", blob.size);
        url = URL.createObjectURL(blob);
        console.log("[useLocalVideo] blob url:", url);
        setBlobUrl(url);
      } catch (e) {
        console.error("[useLocalVideo] failed:", e);
        setError(e);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    load();

    return () => {
      cancelled = true;
      if (url) URL.revokeObjectURL(url);
    };
  }, [videoPath]);

  return { blobUrl, loading, error };
}

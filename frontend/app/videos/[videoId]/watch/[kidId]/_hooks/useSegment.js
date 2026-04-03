import { useEffect, useState, useRef } from "react";

export function useSegments(video_id) {
  const [segments, setSegments] = useState([]);
  const segmentsRef = useRef([]);

  useEffect(() => {
    segmentsRef.current = segments;
  }, [segments]);

  useEffect(() => {
    if (!video_id) return;
    fetch(`${process.env.NEXT_PUBLIC_API_BASE_URL}/api/questions/${video_id}`)
      .then((res) => res.json())
      .then((data) => setSegments(data.segments))
      .catch((err) => console.error(err));
  }, [video_id]);

  return { segments, segmentsRef };
}

import yts from "yt-search";

export async function GET(request) {
  try {
    const { searchParams } = new URL(request.url);
    const q = searchParams.get("q");

    if (!q) {
      return new Response(JSON.stringify({ videos: [] }), {
        headers: {
          "Cache-Control": "public, max-age=3600, stale-while-revalidate=86400", // 1 hour cache + SWR for 1 day
          "Content-Type": "application/json",
        },
      });
    }

    const searchResult = await yts(q);

    const videos = searchResult.videos.map((v) => ({
      id: v.videoId,
      title: v.title,
      thumbnail: v.thumbnail,
      seconds: v.seconds,
    }));

    return new Response(JSON.stringify({ videos }), {
      headers: {
        "Cache-Control": "public, max-age=3600, stale-while-revalidate=86400",
        "Content-Type": "application/json",
      },
    });
  } catch (err) {
    console.error("search error", err);
    return new Response(JSON.stringify({ videos: [] }), {
      headers: {
        "Cache-Control": "public, max-age=3600, stale-while-revalidate=86400",
        "Content-Type": "application/json",
      },
    });
  }
}

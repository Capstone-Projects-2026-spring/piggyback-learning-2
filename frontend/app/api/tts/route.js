import { NextResponse } from 'next/server';

export async function POST(request) {
  try {
    // You can use standard names for Inworld voices (e.g., "Sarah", "Ashley", "Pixie")
    // or pass your own custom cloned voiceId from the Inworld Studio.
    const { text, voiceId = "Pixie" } = await request.json(); 

    if (!text) {
      return NextResponse.json({ error: "Text is required" }, { status: 400 });
    }

    const apiKey = process.env.INWORLD_API_KEY;

    const response = await fetch('https://api.inworld.ai/tts/v1/voice', {
      method: 'POST',
      headers: {
        'Authorization': `Basic ${apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        text: text,
        voiceId: voiceId,
        modelId: "inworld-tts-1.5-max", // Inworld's highest quality voice model
        audioConfig: {
          audioEncoding: "MP3",
          sampleRateHertz: 24000
        }
      }),
    });

    if (!response.ok) {
      const errorData = await response.text();
      throw new Error(`Inworld Error: ${response.status} ${errorData}`);
    }

    const data = await response.json();
    
    // Inworld returns the audio as a base64 string, so we convert it to a binary buffer
    const audioBuffer = Buffer.from(data.audioContent, 'base64');

    return new NextResponse(audioBuffer, {
      headers: {
        'Content-Type': 'audio/mpeg',
      },
    });

  } catch (error) {
    console.error("TTS API Error:", error);
    return NextResponse.json({ error: "Failed to generate speech" }, { status: 500 });
  }
}
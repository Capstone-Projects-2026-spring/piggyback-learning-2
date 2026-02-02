import "https://deno.land/x/xhr@0.1.0/mod.ts";
import { serve } from "https://deno.land/std@0.168.0/http/server.ts";
import { GoogleGenerativeAI } from "https://esm.sh/@google/generative-ai@0.24.1";

const corsHeaders = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Headers": "authorization, x-client-info, apikey, content-type",
};

serve(async (req) => {
  // Handle CORS preflight requests
  if (req.method === "OPTIONS") {
    return new Response(null, { headers: corsHeaders });
  }

  try {
    const { youtubeUrl, customPrompt } = await req.json();

    console.log("Processing request for YouTube URL:", youtubeUrl);

    // Validate input
    if (!youtubeUrl?.trim()) {
      return new Response(
        JSON.stringify({ error: "YouTube URL is required" }),
        {
          status: 400,
          headers: { ...corsHeaders, "Content-Type": "application/json" },
        },
      );
    }

    // Validate YouTube URL format
    const youtubeRegex = /^(https?:\/\/)?(www\.)?(youtube\.com|youtu\.be)\/.+/;
    if (!youtubeRegex.test(youtubeUrl)) {
      return new Response(
        JSON.stringify({ error: "Please enter a valid YouTube URL" }),
        {
          status: 400,
          headers: { ...corsHeaders, "Content-Type": "application/json" },
        },
      );
    }

    // Get API key from environment
    const apiKey = Deno.env.get("GOOGLE_API_KEY") || Deno.env.get("GEMINI_API_KEY");
    if (!apiKey) {
      return new Response(
        JSON.stringify({ error: "API key not configured" }),
        {
          status: 500,
          headers: { ...corsHeaders, "Content-Type": "application/json" },
        },
      );
    }

    // Initialize Gemini 2.5 Pro (multimodal model that can "watch" videos)
    const genAI = new GoogleGenerativeAI(apiKey);
    const model = genAI.getGenerativeModel({ model: "gemini-2.5-pro" });

    // The prompt that instructs Gemini how to create questions
    const prompt = customPrompt || `# Children's Video Comprehension Questions
## Task
Create comprehension questions for children watching videos.

## Process
1. **Analyze video** - Watch completely
2. **Find "Cliffhangers"** - Find timestamps where questions can be asked
3. **Create questions** - Each timestamp 2-3 minutes apart

## Question Types
- **Character**: Who/what they are, appearance, role
- **Setting**: Where/when events happen
- **Feeling**: Emotions shown
- **Action**: What characters did
- **Causal**: Why something happened
- **Outcome**: What resulted from actions
- **Prediction**: What might happen next

## Output Format
Return ONLY JSON:
[
  {
    "question_timestamp": "MM:SS",
    "ques": "Question text here",
    "choices": ["Option 1", "Option 2", "Option 3", "Option 4"],
    "answer": "Correct Answer Here",
    "type": "Character / Setting / Event / Other"
  }
]`;

    // KEY: Pass YouTube URL as fileData so Gemini can analyze the video
    const result = await model.generateContent([
      prompt,
      {
        fileData: { fileUri: youtubeUrl },
      } as any,
    ]);

    const responseText = result.response.text();

    return new Response(
      JSON.stringify({ text: responseText }),
      { headers: { ...corsHeaders, "Content-Type": "application/json" } },
    );
  } catch (error) {
    return new Response(
      JSON.stringify({
        error: error instanceof Error ? error.message : "An unexpected error occurred",
      }),
      { status: 500, headers: { ...corsHeaders, "Content-Type": "application/json" } },
    );
  }
});

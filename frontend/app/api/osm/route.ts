import { NextResponse } from "next/server";
import { promises as fs } from "fs";
import path from "path";

export const runtime = "nodejs";

export async function GET() {
  try {
    const filePath = path.resolve(process.cwd(), "..", "out.json");
    const raw = await fs.readFile(filePath, "utf-8");
    const json = JSON.parse(raw);
    return NextResponse.json(json);
  } catch (err) {
    return NextResponse.json(
      {
        error: "Failed to read out.json",
        message: err instanceof Error ? err.message : String(err),
      },
      { status: 500 }
    );
  }
}


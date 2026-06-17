import { invoke } from "@tauri-apps/api/core";

export type ExportNotesRequest = {
  video_path: string;
  note_ids: string[];
  output_dir: string;
  filename: string;
  include_screenshots: boolean;
  group_by_chapters: boolean;
};

export type ExportNotesResult = {
  markdown_path: string;
  assets_dir: string;
  note_count: number;
  screenshot_count: number;
};

export async function exportNotesToMarkdown(
  request: ExportNotesRequest,
): Promise<ExportNotesResult> {
  return invoke("export_notes_to_markdown", { request });
}

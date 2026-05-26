import type { NoteType } from "../../lib/tauri";
import {
  renderStudyFrontHtml,
  studyOrdinalsForNote,
} from "../../lib/studyPreview";

export function NoteFrontPreview({
  noteType,
  fields,
  cardCount,
}: {
  noteType: NoteType;
  fields: Record<string, string>;
  cardCount: number;
}) {
  const ordinals = studyOrdinalsForNote(noteType, fields, cardCount);

  return (
    <div className="space-y-2">
      <style>{noteType.css}</style>
      {ordinals.map((ordinal) => (
        <div
          key={ordinal}
          className="text-sm text-text leading-relaxed whitespace-pre-wrap prose prose-sm prose-stone dark:prose-invert max-w-none"
          dangerouslySetInnerHTML={{
            __html: renderStudyFrontHtml(noteType, fields, ordinal),
          }}
        />
      ))}
    </div>
  );
}

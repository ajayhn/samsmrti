import type { ReviewCard } from "../../lib/tauri";
import { NoteEditorDialog } from "../notes/NoteEditorDialog";

interface ReviewNoteEditorProps {
  card: ReviewCard;
  open: boolean;
  onClose: () => void;
  onSaved: (fields: Record<string, string>) => void;
}

export function ReviewNoteEditor({
  card,
  open,
  onClose,
  onSaved,
}: ReviewNoteEditorProps) {
  return (
    <NoteEditorDialog
      note={{
        note_id: card.note_id,
        note_type_id: card.note_type_id,
        fields: card.fields,
      }}
      open={open}
      onClose={onClose}
      onSaved={(fields) => onSaved(fields)}
    />
  );
}

import { useContentTransferStore } from "../../stores/contentTransferStore";
import { ContentDeckPicker } from "./ContentDeckPicker";

export function ContentTransferHost() {
  const picker = useContentTransferStore((s) => s.picker);
  const closePicker = useContentTransferStore((s) => s.closePicker);

  if (!picker) return null;

  const { mode, decks, resolve } = picker;

  const title = mode === "export" ? "Export content" : "Import content";
  const description =
    mode === "export"
      ? "Choose decks to include in the export file. Subdecks of a selected deck are included automatically."
      : "Choose decks to import from this file. Subdecks of a selected deck are included automatically.";
  const confirmLabel = mode === "export" ? "Export" : "Import";

  return (
    <ContentDeckPicker
      title={title}
      description={description}
      decks={decks}
      confirmLabel={confirmLabel}
      onCancel={() => {
        resolve(null);
        closePicker();
      }}
      onConfirm={(deckIds) => {
        resolve(deckIds);
        closePicker();
      }}
    />
  );
}

import { confirm, message, open, save } from "@tauri-apps/plugin-dialog";
import { api } from "./tauri";
import { useDeckStore } from "../stores/deckStore";
import { useKarmaStore } from "../stores/karmaStore";
import { useProfileStore } from "../stores/profileStore";

async function refreshAfterDataChange() {
  await useDeckStore.getState().fetchDecks();
  await useProfileStore.getState().fetchProfiles();
  await useKarmaStore.getState().fetchKarma();
}

export async function exportContentJson(): Promise<void> {
  const timestamp = new Date().toISOString().slice(0, 10);
  const filePath = await save({
    defaultPath: `samsmrti-content-${timestamp}.json`,
    filters: [{ name: "JSON", extensions: ["json"] }],
  });
  if (!filePath) return;

  try {
    const result = await api.exportContentJson(filePath);
    await message(
      `Exported ${result.decks} decks, ${result.notes} notes, ${result.cards} cards, ${result.entities} entities, and ${result.triples} triples.\n\nStudy history, profiles, and karma were not included. Import this on another machine so each user starts with fresh scheduling.`,
      { title: "Content export complete", kind: "info" }
    );
  } catch (e) {
    await message(`Export failed: ${e}`, { title: "Export failed", kind: "error" });
  }
}

export async function importContentJson(): Promise<void> {
  const ok = await confirm(
    "Import content from a JSON file?\n\nExisting rows with the same IDs are kept; only new decks, notes, cards, and graph data are added. All imported cards start as \"new\" for every profile.",
    { title: "Import content", kind: "warning" }
  );
  if (!ok) return;

  const filePath = await open({
    multiple: false,
    filters: [
      { name: "Samsmrti content", extensions: ["json", "gz", "json.gz"] },
    ],
  });
  if (!filePath || Array.isArray(filePath)) return;

  try {
    const result = await api.importContentJson(filePath);
    await refreshAfterDataChange();
    const warn =
      result.warnings.length > 0
        ? `\n\nWarnings:\n${result.warnings.slice(0, 5).join("\n")}`
        : "";
    await message(
      `Added ${result.decks_added} decks, ${result.notes_added} notes, ${result.cards_added} cards, ${result.entities_added} entities, ${result.triples_added} triples. Skipped ${result.rows_skipped} existing rows.${warn}`,
      { title: "Content import complete", kind: "info" }
    );
  } catch (e) {
    await message(`Import failed: ${e}`, { title: "Import failed", kind: "error" });
  }
}

export async function exportFullBackup(): Promise<void> {
  const timestamp = new Date().toISOString().slice(0, 10);
  const filePath = await save({
    defaultPath: `samsmrti-full-${timestamp}.samsmrti-backup`,
    filters: [
      { name: "Samsmrti backup", extensions: ["samsmrti-backup", "zip"] },
    ],
  });
  if (!filePath) return;

  try {
    const result = await api.exportFullBackup(filePath);
    const mb = (result.bytes_written / (1024 * 1024)).toFixed(2);
    await message(
      `Full backup saved (${mb} MB, ${result.media_files} media files).\n\nIncludes database, all profiles, review history, karma, scheduling, and media.`,
      { title: "Backup complete", kind: "info" }
    );
  } catch (e) {
    await message(`Backup failed: ${e}`, { title: "Backup failed", kind: "error" });
  }
}

export async function restoreFullBackup(): Promise<void> {
  const ok = await confirm(
    "Restore a full backup?\n\nThis replaces your entire database and media folder. Your current database is copied to samsmrti.db.pre-restore-<timestamp> in the app data folder before overwrite.",
    { title: "Restore full backup", kind: "warning" }
  );
  if (!ok) return;

  const filePath = await open({
    multiple: false,
    filters: [
      { name: "Samsmrti backup", extensions: ["samsmrti-backup", "zip"] },
    ],
  });
  if (!filePath || Array.isArray(filePath)) return;

  try {
    const result = await api.restoreFullBackup(filePath);
    await refreshAfterDataChange();
    const prev = result.previous_db_backup
      ? `\n\nPrevious DB saved to:\n${result.previous_db_backup}`
      : "";
    await message(
      `Restored ${result.decks} decks, ${result.notes} notes, ${result.cards} cards, ${result.profiles} profiles, ${result.media_files_restored} media files.${prev}`,
      { title: "Restore complete", kind: "info" }
    );
  } catch (e) {
    await message(`Restore failed: ${e}`, { title: "Restore failed", kind: "error" });
  }
}

/** @deprecated Use exportContentJson — kept for older menu references */
export async function exportLegacyGzBackup(): Promise<void> {
  const timestamp = new Date().toISOString().slice(0, 10);
  const filePath = await save({
    defaultPath: `samsmrti-backup-${timestamp}.json.gz`,
    filters: [{ name: "Gzipped JSON", extensions: ["json.gz", "gz"] }],
  });
  if (!filePath) return;
  try {
    const result = await api.exportAllGz(filePath);
    const kb = (result.bytes_written / 1024).toFixed(1);
    await message(
      `Exported ${result.decks} decks, ${result.notes} notes (${kb} KB gzipped). Same content scope as Export Content — use .json export when sharing.`,
      { title: "Export complete", kind: "info" }
    );
  } catch (e) {
    await message(`Export failed: ${e}`, { title: "Export failed", kind: "error" });
  }
}

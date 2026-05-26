/** Let the OS / WebView handle copy, paste, cut, and select-all. */
export function isNativeEditShortcut(e: KeyboardEvent): boolean {
  if (!(e.metaKey || e.ctrlKey) || e.altKey) return false;
  const k = e.key.toLowerCase();
  return k === "c" || k === "v" || k === "x" || k === "a";
}

export function hasTextSelection(): boolean {
  const sel = window.getSelection();
  return (sel?.toString().trim().length ?? 0) > 0;
}

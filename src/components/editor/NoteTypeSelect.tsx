import { useEffect, useMemo, useRef, useState } from "react";
import { filterInputProps } from "../../lib/filterInput";
import type { NoteType } from "../../lib/tauri";

interface Props {
  noteTypes: NoteType[];
  value: string;
  onChange: (noteType: NoteType) => void;
}

export function NoteTypeSelect({ noteTypes, value, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);

  const selected = noteTypes.find((t) => t.id === value);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return noteTypes;
    return noteTypes.filter((t) => t.name.toLowerCase().includes(q));
  }, [noteTypes, query]);

  useEffect(() => {
    if (!open) {
      setQuery(selected?.name ?? "");
    }
  }, [open, selected?.name]);

  useEffect(() => {
    const onDocClick = (e: MouseEvent) => {
      if (!containerRef.current?.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, []);

  const pick = (nt: NoteType) => {
    onChange(nt);
    setQuery(nt.name);
    setOpen(false);
  };

  return (
    <div ref={containerRef} className="relative">
      <input
        type="text"
        value={open ? query : selected?.name ?? query}
        onChange={(e) => {
          setQuery(e.target.value);
          setOpen(true);
        }}
        onFocus={() => {
          setQuery(selected?.name ?? "");
          setOpen(true);
        }}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            setOpen(false);
            setQuery(selected?.name ?? "");
          } else if (e.key === "Enter" && filtered.length > 0) {
            e.preventDefault();
            pick(filtered[0]);
          }
        }}
        placeholder="Search note types..."
        className="w-full px-4 py-2.5 bg-surface-alt border border-border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
        role="combobox"
        aria-expanded={open}
        aria-autocomplete="list"
        name="samsmrti-note-type-combobox"
        {...filterInputProps}
      />
      {open && (
        <ul
          className="absolute z-20 mt-1 w-full max-h-56 overflow-y-auto bg-surface border border-border rounded-xl shadow-lg py-1"
          role="listbox"
        >
          {filtered.length === 0 ? (
            <li className="px-4 py-2 text-sm text-text-muted">No matching note types</li>
          ) : (
            filtered.map((nt) => (
              <li key={nt.id}>
                <button
                  type="button"
                  onClick={() => pick(nt)}
                  className={`w-full text-left px-4 py-2 text-sm hover:bg-surface-alt cursor-pointer ${
                    nt.id === value ? "text-primary-600 font-medium" : "text-text"
                  }`}
                  role="option"
                  aria-selected={nt.id === value}
                >
                  {nt.name}
                  {nt.is_cloze && (
                    <span className="ml-2 text-xs text-text-muted">cloze</span>
                  )}
                </button>
              </li>
            ))
          )}
        </ul>
      )}
    </div>
  );
}

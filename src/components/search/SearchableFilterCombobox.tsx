import { useEffect, useMemo, useRef, useState } from "react";
import { filterInputProps } from "../../lib/filterInput";

export interface FilterOption {
  value: string;
  label: string;
  hint?: string;
  depth?: number;
}

interface Props {
  options: FilterOption[];
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
  clearLabel?: string;
  className?: string;
  inputName?: string;
  ariaLabel?: string;
}

function matchesQuery(label: string, q: string): boolean {
  const lower = label.toLowerCase();
  if (lower.includes(q)) return true;
  return label.split("::").some((part) => part.toLowerCase().includes(q));
}

export function SearchableFilterCombobox({
  options,
  value,
  onChange,
  placeholder,
  clearLabel = "All",
  className = "",
  inputName,
  ariaLabel,
}: Props) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);

  const selected = options.find((o) => o.value === value);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    const pool = q
      ? options.filter((o) => matchesQuery(o.label, q))
      : options;
    return pool.slice(0, 50);
  }, [options, query]);

  useEffect(() => {
    if (!open) {
      setQuery(selected?.label ?? "");
    }
  }, [open, selected?.label]);

  useEffect(() => {
    const onDocClick = (e: MouseEvent) => {
      if (!containerRef.current?.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, []);

  const pick = (opt: FilterOption | null) => {
    onChange(opt?.value ?? "");
    setQuery(opt?.label ?? "");
    setOpen(false);
  };

  const displayValue = open ? query : (selected?.label ?? "");

  return (
    <div ref={containerRef} className={`relative ${className}`}>
      <input
        type="text"
        value={displayValue}
        placeholder={placeholder}
        onChange={(e) => {
          setQuery(e.target.value);
          setOpen(true);
        }}
        onFocus={() => {
          setQuery(selected?.label ?? "");
          setOpen(true);
        }}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            setOpen(false);
            setQuery(selected?.label ?? "");
          } else if (e.key === "Enter") {
            e.preventDefault();
            if (filtered.length > 0) {
              pick(filtered[0]);
            } else if (!query.trim()) {
              pick(null);
            }
          } else if (e.key === "Backspace" && !query && value) {
            e.preventDefault();
            pick(null);
          }
        }}
        className="w-full px-3 py-2.5 bg-surface-alt border border-border rounded-xl text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
        role="combobox"
        aria-expanded={open}
        aria-autocomplete="list"
        aria-label={ariaLabel}
        name={inputName}
        {...filterInputProps}
      />
      {open && (
        <ul
          className="absolute z-20 mt-1 w-full min-w-[220px] max-h-56 overflow-y-auto bg-surface border border-border rounded-xl shadow-lg py-1"
          role="listbox"
        >
          <li>
            <button
              type="button"
              onClick={() => pick(null)}
              className={`w-full text-left px-3 py-2 text-sm hover:bg-surface-alt cursor-pointer ${
                !value ? "text-primary-600 font-medium" : "text-text-muted"
              }`}
              role="option"
              aria-selected={!value}
            >
              {clearLabel}
            </button>
          </li>
          {filtered.length === 0 ? (
            <li className="px-3 py-2 text-sm text-text-muted">No matches</li>
          ) : (
            filtered.map((opt) => (
              <li key={opt.value}>
                <button
                  type="button"
                  onClick={() => pick(opt)}
                  className={`w-full text-left py-2 text-sm hover:bg-surface-alt cursor-pointer flex items-center justify-between gap-2 min-w-0 ${
                    opt.value === value
                      ? "text-primary-600 font-medium"
                      : "text-text"
                  }`}
                  style={{ paddingLeft: `${12 + (opt.depth ?? 0) * 14}px`, paddingRight: 12 }}
                  role="option"
                  aria-selected={opt.value === value}
                >
                  <span className="truncate">{opt.label}</span>
                  {opt.hint != null && opt.hint !== "" && (
                    <span className="text-xs text-text-muted shrink-0 tabular-nums">
                      {opt.hint}
                    </span>
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

import type { InputHTMLAttributes } from "react";

/**
 * Disables browser spellcheck, autocorrect, and autofill on inputs that filter
 * in-app lists (decks, tags, note types, etc.). App UI supplies suggestions only.
 */
export const filterInputProps: Pick<
  InputHTMLAttributes<HTMLInputElement>,
  "autoComplete" | "autoCorrect" | "autoCapitalize" | "spellCheck"
> & {
  "data-lpignore"?: string;
  "data-1p-ignore"?: string;
  "data-form-type"?: string;
} = {
  autoComplete: "off",
  autoCorrect: "off",
  autoCapitalize: "off",
  spellCheck: false,
  "data-lpignore": "true",
  "data-1p-ignore": "true",
  "data-form-type": "other",
};

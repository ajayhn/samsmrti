import { Mark, mergeAttributes } from "@tiptap/react";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

const CLOZE_RE = /\{\{c(\d+)::([^}]*?)(?:::([^}]*?))?\}\}/g;

export const ClozeHighlight = Mark.create({
  name: "clozeHighlight",

  addOptions() {
    return {
      HTMLAttributes: {},
    };
  },

  parseHTML() {
    return [{ tag: "span.cloze-mark" }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes, {
        class: "cloze-mark",
      }),
      0,
    ];
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey("clozeDecorations"),
        props: {
          decorations(state) {
            const decorations: Decoration[] = [];
            const doc = state.doc;

            doc.descendants((node, pos) => {
              if (!node.isText || !node.text) return;

              let match;
              CLOZE_RE.lastIndex = 0;
              while ((match = CLOZE_RE.exec(node.text)) !== null) {
                const start = pos + match.index;
                const end = start + match[0].length;
                decorations.push(
                  Decoration.inline(start, end, {
                    class: "cloze-highlight",
                  })
                );
              }
            });

            return DecorationSet.create(doc, decorations);
          },
        },
      }),
    ];
  },
});

export function insertCloze(
  editor: { chain: () => any; state: { doc: any } },
  existingText?: string
) {
  const doc = editor.state.doc;
  let maxN = 0;

  doc.descendants((node: { isText: boolean; text?: string }) => {
    if (!node.isText || !node.text) return;
    let m;
    CLOZE_RE.lastIndex = 0;
    while ((m = CLOZE_RE.exec(node.text)) !== null) {
      const n = parseInt(m[1], 10);
      if (n > maxN) maxN = n;
    }
  });

  const nextN = maxN + 1;
  const text = existingText || "...";
  const clozeStr = `{{c${nextN}::${text}}}`;

  editor.chain().focus().insertContent(clozeStr).run();
}

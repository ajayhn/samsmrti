import { Node, mergeAttributes } from "@tiptap/react";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

const WIKI_LINK_RE = /\[\[([^\]]+)\]\]/g;

export const WikiLink = Node.create({
  name: "wikiLink",
  group: "inline",
  inline: true,
  atom: true,

  addAttributes() {
    return {
      target: { default: null },
    };
  },

  parseHTML() {
    return [{ tag: 'span[data-wiki-link]' }];
  },

  renderHTML({ node, HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(HTMLAttributes, {
        "data-wiki-link": node.attrs.target,
        class: "wiki-link",
      }),
      `[[${node.attrs.target}]]`,
    ];
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey("wikiLinkDecorations"),
        props: {
          decorations(state) {
            const decorations: Decoration[] = [];
            const doc = state.doc;

            doc.descendants((node, pos) => {
              if (!node.isText || !node.text) return;

              let match;
              WIKI_LINK_RE.lastIndex = 0;
              while ((match = WIKI_LINK_RE.exec(node.text)) !== null) {
                const start = pos + match.index;
                const end = start + match[0].length;
                decorations.push(
                  Decoration.inline(start, end, {
                    class: "wiki-link-highlight",
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

export function extractWikiLinks(text: string): string[] {
  const links: string[] = [];
  let match;
  WIKI_LINK_RE.lastIndex = 0;
  while ((match = WIKI_LINK_RE.exec(text)) !== null) {
    links.push(match[1]);
  }
  return [...new Set(links)];
}

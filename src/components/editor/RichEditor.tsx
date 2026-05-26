import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Image from "@tiptap/extension-image";
import Placeholder from "@tiptap/extension-placeholder";
import { ClozeHighlight, insertCloze } from "./ClozeExtension";
import { WikiLink } from "./WikiLinkExtension";

interface RichEditorProps {
  content: string;
  onChange: (html: string, text: string) => void;
  placeholder?: string;
  isCloze?: boolean;
  className?: string;
}

export function RichEditor({
  content,
  onChange,
  placeholder,
  isCloze,
  className,
}: RichEditorProps) {
  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        heading: { levels: [1, 2, 3] },
      }),
      Image.configure({
        inline: true,
        allowBase64: true,
      }),
      Placeholder.configure({
        placeholder: placeholder || "Start typing...",
      }),
      ClozeHighlight,
      WikiLink,
    ],
    content,
    onUpdate: ({ editor }) => {
      onChange(editor.getHTML(), editor.getText());
    },
    editorProps: {
      attributes: {
        class:
          "prose prose-stone dark:prose-invert prose-sm max-w-none focus:outline-none min-h-[100px] px-4 py-3",
      },
      handleKeyDown: (_view, event) => {
        if (
          isCloze &&
          event.ctrlKey &&
          event.shiftKey &&
          event.key === "C"
        ) {
          event.preventDefault();
          if (editor) {
            const { from, to } = editor.state.selection;
            if (from !== to) {
              const selectedText = editor.state.doc.textBetween(from, to);
              editor
                .chain()
                .focus()
                .deleteSelection()
                .run();
              insertCloze(editor, selectedText);
            } else {
              insertCloze(editor);
            }
          }
          return true;
        }
        return false;
      },
      handleDrop: (_view, event) => {
        const files = event.dataTransfer?.files;
        if (files && files.length > 0) {
          event.preventDefault();
          Array.from(files).forEach((file) => {
            if (file.type.startsWith("image/")) {
              const reader = new FileReader();
              reader.onload = (e) => {
                const src = e.target?.result as string;
                editor?.chain().focus().setImage({ src }).run();
              };
              reader.readAsDataURL(file);
            }
          });
          return true;
        }
        return false;
      },
      handlePaste: (_view, event) => {
        const items = event.clipboardData?.items;
        if (items) {
          for (const item of Array.from(items)) {
            if (item.type.startsWith("image/")) {
              event.preventDefault();
              const file = item.getAsFile();
              if (file) {
                const reader = new FileReader();
                reader.onload = (e) => {
                  const src = e.target?.result as string;
                  editor?.chain().focus().setImage({ src }).run();
                };
                reader.readAsDataURL(file);
              }
              return true;
            }
          }
        }
        return false;
      },
    },
  });

  return (
    <div
      className={`border border-border rounded-xl bg-surface-alt overflow-hidden ${className || ""}`}
    >
      {/* Toolbar */}
      <div className="flex items-center gap-0.5 px-2 py-1.5 border-b border-border bg-surface">
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleBold().run()}
          active={editor?.isActive("bold")}
          title="Bold (Ctrl+B)"
        >
          B
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleItalic().run()}
          active={editor?.isActive("italic")}
          title="Italic (Ctrl+I)"
        >
          <em>I</em>
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleCode().run()}
          active={editor?.isActive("code")}
          title="Inline Code"
        >
          <code className="text-xs">&lt;/&gt;</code>
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleBulletList().run()}
          active={editor?.isActive("bulletList")}
          title="Bullet List"
        >
          &bull;
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleOrderedList().run()}
          active={editor?.isActive("orderedList")}
          title="Numbered List"
        >
          1.
        </ToolbarButton>

        <div className="w-px h-5 bg-border mx-1" />

        {isCloze && (
          <ToolbarButton
            onClick={() => editor && insertCloze(editor)}
            title="Insert Cloze (Ctrl+Shift+C)"
          >
            <span className="text-xs font-mono">[...]</span>
          </ToolbarButton>
        )}

        <ToolbarButton
          onClick={() => {
            const url = prompt("Image URL:");
            if (url) editor?.chain().focus().setImage({ src: url }).run();
          }}
          title="Insert Image"
        >
          <span className="text-xs">IMG</span>
        </ToolbarButton>
      </div>

      <EditorContent editor={editor} />
    </div>
  );
}

function ToolbarButton({
  onClick,
  active,
  title,
  children,
}: {
  onClick: () => void;
  active?: boolean;
  title?: string;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={title}
      className={`px-2 py-1 rounded text-sm font-medium transition-colors cursor-pointer ${
        active
          ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300"
          : "text-text-secondary hover:bg-surface-hover hover:text-text"
      }`}
    >
      {children}
    </button>
  );
}

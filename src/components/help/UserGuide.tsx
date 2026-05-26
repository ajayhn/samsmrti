import React, { useState } from "react";

const SECTIONS = [
  {
    title: "Getting Started",
    content: `
**Welcome to Samsmrti!** A modern spaced repetition app for effective learning.

Samsmrti comes with four example decks (Science, Math, History, Geography) to get you started right away. Click any deck in the sidebar to see its cards, then hit **Study Now** to begin reviewing.

**Quick Start:**
1. Select a deck from the sidebar
2. Click **Study Now** to start a review session
3. Rate each card: Again (1), Hard (2), Good (3), or Easy (4)
4. The algorithm will schedule cards at optimal intervals
`,
  },
  {
    title: "Backup, Export & Import",
    content: `
**Two modes** (also in **Settings → Data** and the **File** menu):

**1. Content (JSON)** — share decks; recipient starts fresh
- **Export Content** → \`samsmrti-content-YYYY-MM-DD.json\`
- **Import Content** → merges new IDs; imported cards are **new** for every profile
- Includes: decks, note types, notes, cards, tags, knowledge graph
- Excludes: review log, per-profile scheduling, profiles, karma, media binaries (copy the app **media** folder separately if needed)

**2. Full backup (binary .samsmrti-backup)** — move to another computer
- **Backup (Full)** → zip with database + media + all study state
- **Restore (Full)** → replaces local data (previous DB saved as \`samsmrti.db.pre-restore-*\`)

**Single deck:** Settings → Export Deck (JSON, one deck).

See **QUICKSTART.md** in the project for paths and manual copy instructions.
`,
  },
  {
    title: "Creating Cards",
    content: `
**Adding new cards:**
1. Select a deck, then click **Add Cards**
2. Choose a note type (Basic, Basic & Reversed, or Cloze)
3. Fill in the fields using the rich text editor
4. Add tags to organize your cards
5. Click **Create Note**

**Editor features:**
- **Rich text**: Bold, italic, code, lists
- **Images**: Drag and drop, paste from clipboard, or insert URL
- **Cloze deletions**: Select text and press \`Ctrl+Shift+C\` or use the toolbar button
- **Wiki-links**: Type \`[[card title]]\` to link to other cards

**Note Types:**
- **Basic**: Front and Back fields, one card generated
- **Basic (and reversed)**: Creates two cards — Front→Back and Back→Front
- **Cloze**: Use \`{{c1::answer}}\` syntax. Each cloze number generates a separate card
`,
  },
  {
    title: "Reviewing Cards",
    content: `
**During a review session:**
- Press **Space** to reveal the answer
- Rate with keyboard: **1** (Again), **2** (Hard), **3** (Good), **4** (Easy)
- Press **Escape** to end the session early
- Press **9** to bury the card (hidden until tomorrow)
- Press **D** to delete the card (removes it from the deck; use for wrong or redundant cards)
- Press **U** to undo the last action (rating, bury, or delete)
- Press **Ctrl+Z** also undoes the last action
- Press **E** (or use **Edit** in the header) to edit the current note’s fields without leaving the session

**Buried cards:**
Open **Browse Cards** → **Buried** tab to search and unbury cards early.

**Countries deck & Knowledge Map:**
Country Details notes automatically create entities (Country, City, River, etc.) and triples (Has-River, Capital, On-Continent, …). Each study card is linked to its triple so you can review from the Knowledge Map by entity.

**How scheduling works:**
Samsmrti uses a spaced repetition algorithm based on FSRS. Each card has:
- **Stability**: How long until you might forget it
- **Difficulty**: How inherently hard it is for you

Rating effects:
- **Again**: Card is marked as a lapse, shown again very soon
- **Hard**: Interval increases slightly
- **Good**: Interval increases significantly (recommended for most cards)
- **Easy**: Interval increases substantially

**Hierarchical decks (like Anki):**
- Create **subdecks** with **Create Subdeck** on any deck page
- Sidebar shows nested deck trees
- **Study Now** on a parent deck includes cards from all subdecks
- Card counts on parent decks roll up from subdecks
- **Deck Settings** → **Parent deck** to move a deck or make it top-level
- Deleting a deck removes its subdecks too

**Deck Settings:**
Click **Settings** on any deck to adjust:
- Parent deck (for hierarchy)
- New cards per day (default: 20)
- Maximum reviews per day (default: 200)
`,
  },
  {
    title: "Importing Decks",
    content: `
Samsmrti can import decks from other flashcard apps:

**Anki (.apkg):**
1. In Anki, go to File → Export
2. Choose "Anki Deck Package (.apkg)"
3. In Samsmrti, click **Import Deck** in the sidebar
4. Select the .apkg file

**Mochi (.mochi):**
1. In Mochi, export your deck
2. In Samsmrti, click **Import Deck** in the sidebar
3. Select the .mochi file

**Anki collection file:** Quit Anki, then import **collection.anki2** from **~/Library/Application Support/Anki2/** (your profile folder, e.g. Combined) for your entire library at once.

Both formats will import:
- Deck structure and hierarchy (Anki Parent::Child paths become nested decks)
- Notes and their fields
- Card scheduling state
- Tags
- Media files (images)
`,
  },
  {
    title: "Searching & Browsing",
    content: `
**Card Browser:**
Click **Browse Cards** in the sidebar to search across all your cards.

- Type any text to search by content
- Filter by deck using the dropdown
- **Tags tab** (Browse → **Tags**): browse all imported Anki tags by name; click a tag to see its notes
- Filter by tag on the Search tab (dropdown)
- Click any result to expand and see all fields

The search uses full-text search (FTS5), so it works with partial words and is fast even with large collections.
`,
  },
  {
    title: "Knowledge Graph",
    content: `
**Visualize connections between your cards:**

Click **Knowledge Graph** in the sidebar to see an interactive visualization.

Cards are connected by:
- **Wiki-links**: When you type \`[[card title]]\` in a card, it creates an explicit link
- **Shared tags**: Cards with the same tag are connected through tag nodes

You can:
- Drag nodes to rearrange the layout
- Filter by deck or view the global graph
- Purple nodes are cards, amber nodes are tags
`,
  },
  {
    title: "Profiles & Karma",
    content: `
**Profiles** (honor system, no password): Use the profile switcher above Settings to pick who is studying. Each profile has their own review schedule and stats on the shared card library — switching profiles when someone else uses Samsmrti keeps due counts and Karma fair. Create profiles in Settings or the sidebar switcher.

**Admin** is a built-in profile for maintenance — it never earns Karma.

**Karma** appears as a dollar counter in the top-right. You earn **$0.10** per card reviewed and **$0.20** per card added (2× for adding). A **qualifying day** needs 10+ minutes of active study *or* 15+ effective actions (reviews count 1, adds count 2). Consistent qualifying days build a **streak**; every **7 qualifying days** earns a **$5** bonus. Binge reviewing past 50 cards/day earns less per card — steady daily study is rewarded more.
`,
  },
  {
    title: "Statistics",
    content: `
Click **Statistics** in the sidebar to see:

- **Card distribution**: How many cards are New, Learning, or in Review
- **Daily activity**: A chart of your last 30 days of reviews
- **Review streak**: Consecutive days with at least one review (shared collection)
- **Karma**: Balance, karma streak, and qualifying days for the active profile
- **Rating breakdown**: How many times you rated Again, Hard, Good, or Easy

Use stats to monitor your progress and identify areas that need more practice.
`,
  },
  {
    title: "Keyboard Shortcuts",
    content: `
| Action | Shortcut |
|--------|----------|
| Show answer | Space |
| Rate Again | 1 |
| Rate Hard | 2 |
| Rate Good | 3 |
| Rate Easy | 4 |
| Undo last rating | Ctrl/Cmd + Z |
| End session | Escape |
| Insert cloze | Ctrl/Cmd + Shift + C |
`,
  },
];

export function UserGuide() {
  const [activeSection, setActiveSection] = useState(0);

  return (
    <div className="h-full flex overflow-hidden">
      {/* TOC sidebar */}
      <div className="w-56 border-r border-border p-3 overflow-y-auto shrink-0">
        <h3 className="text-xs font-semibold text-text-muted uppercase tracking-wider mb-3 px-2">
          User Guide
        </h3>
        {SECTIONS.map((section, i) => (
          <button
            key={i}
            onClick={() => setActiveSection(i)}
            className={`w-full text-left px-3 py-2 text-sm rounded-lg transition-colors cursor-pointer ${
              activeSection === i
                ? "bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-300 font-medium"
                : "text-text-secondary hover:bg-surface-hover"
            }`}
          >
            {section.title}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-8">
        <h2 className="text-2xl font-bold text-text mb-6">
          {SECTIONS[activeSection].title}
        </h2>
        <div className="prose prose-stone dark:prose-invert max-w-2xl">
          <MarkdownRenderer content={SECTIONS[activeSection].content} />
        </div>
      </div>
    </div>
  );
}

function MarkdownRenderer({ content }: { content: string }) {
  const lines = content.trim().split("\n");
  const elements: React.JSX.Element[] = [];
  let key = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    if (line.startsWith("| ") && lines[i + 1]?.startsWith("|--")) {
      const headers = line.split("|").filter(Boolean).map((h) => h.trim());
      const rows: string[][] = [];
      i += 2;
      while (i < lines.length && lines[i].startsWith("| ")) {
        rows.push(lines[i].split("|").filter(Boolean).map((c) => c.trim()));
        i++;
      }
      i--;
      elements.push(
        <table key={key++} className="text-sm">
          <thead>
            <tr>
              {headers.map((h, j) => (
                <th key={j} className="text-left px-3 py-2 border-b border-border">
                  {h}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row, j) => (
              <tr key={j}>
                {row.map((cell, k) => (
                  <td key={k} className="px-3 py-2 border-b border-border">
                    <InlineMarkdown text={cell} />
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      );
    } else if (line.startsWith("- ")) {
      const items: string[] = [line.slice(2)];
      while (i + 1 < lines.length && lines[i + 1].startsWith("- ")) {
        i++;
        items.push(lines[i].slice(2));
      }
      elements.push(
        <ul key={key++} className="list-disc pl-5 space-y-1 text-sm text-text-secondary">
          {items.map((item, j) => (
            <li key={j}>
              <InlineMarkdown text={item} />
            </li>
          ))}
        </ul>
      );
    } else if (/^\d+\. /.test(line)) {
      const items: string[] = [line.replace(/^\d+\. /, "")];
      while (i + 1 < lines.length && /^\d+\. /.test(lines[i + 1])) {
        i++;
        items.push(lines[i].replace(/^\d+\. /, ""));
      }
      elements.push(
        <ol key={key++} className="list-decimal pl-5 space-y-1 text-sm text-text-secondary">
          {items.map((item, j) => (
            <li key={j}>
              <InlineMarkdown text={item} />
            </li>
          ))}
        </ol>
      );
    } else if (line.trim() === "") {
      elements.push(<div key={key++} className="h-3" />);
    } else {
      elements.push(
        <p key={key++} className="text-sm text-text-secondary leading-relaxed">
          <InlineMarkdown text={line} />
        </p>
      );
    }
  }

  return <>{elements}</>;
}

function InlineMarkdown({ text }: { text: string }) {
  const parts = text.split(/(\*\*[^*]+\*\*|`[^`]+`)/g);
  return (
    <>
      {parts.map((part, i) => {
        if (part.startsWith("**") && part.endsWith("**")) {
          return (
            <strong key={i} className="font-semibold text-text">
              {part.slice(2, -2)}
            </strong>
          );
        }
        if (part.startsWith("`") && part.endsWith("`")) {
          return (
            <code
              key={i}
              className="px-1.5 py-0.5 bg-surface-alt rounded text-xs font-mono text-primary-600 dark:text-primary-400"
            >
              {part.slice(1, -1)}
            </code>
          );
        }
        return <span key={i}>{part}</span>;
      })}
    </>
  );
}

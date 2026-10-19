// Mirrors the repository's Markdown into the site's content collection.
//
// The specification, the ldf-mcp guide, the methodology, the design notes,
// the research notes, the measured report, the roadmap and AGENTS.md stay
// where they are, written as plain Markdown for GitHub; this script copies
// them into src/content/docs/docs with the frontmatter Starlight needs and
// links rewritten to site paths. The copies are generated and gitignored.
// Runs under bun with nothing but the standard library.
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..");
const out = join(here, "..", "src", "content", "docs");
const github = "https://github.com/caioricciuti/ldf/blob/main/";

// source path (from the repository root) -> site slug, description, and a
// title where the file's own H1 is not the right page name
const pages = [
  ["docs/ldf-spec-v0.md", "docs/spec", "Every rule of the format, checked by the reference implementation's tests.", "Specification (v0.2)"],
  ["docs/agents.md", "docs/agents", "LDF with Claude Code, Codex CLI and OpenCode: setup, what is intercepted, limits.", "Coding agents"],
  ["docs/ldf-mcp.md", "docs/mcp", "LDF between a data source and the model: the proxy and the ClickHouse mode.", "ldf-mcp"],
  ["docs/benchmark-methodology.md", "docs/methodology", "How every number on this site was produced, and what it does not show."],
  ["docs/design-candidates.md", "docs/design", "The candidates that were measured and the decisions they led to."],
  ["docs/research.md", "docs/research", "What exists already, with facts established by implementing and measuring it."],
  ["docs/reports/2026-10-17-baseline/README.md", "docs/results/baseline", "Perf, tokens and comprehension runs on an M2 Pro, with the raw files next to them.", "Baseline report (October 2026)"],
  ["docs/roadmap.md", "docs/roadmap", "What is done, what is next, what is deliberately not done yet."],
  ["AGENTS.md", "docs/working-notes", "Architecture, commands and conventions for anyone touching the repository.", "Working notes"],
  ["CHANGELOG.md", "docs/changelog", "What changed, by version of the specification."],
  ["CONTRIBUTING.md", "docs/contributing", "The bar for a change: measurements first, fixtures with the spec, pinned dependencies."],
];

// The README is not mirrored: the hand-written overview under /docs/
// replaces it on the site, so links to it land there.
const slugFor = new Map([...pages.map(([path, slug]) => [path, slug]), ["README.md", "docs"]]);

function rewriteLinks(markdown, fromPath) {
  const fromDir = fromPath.includes("/") ? dirname(fromPath) : "";
  return markdown.replace(/\]\(([^)\s]+)\)/g, (whole, target) => {
    if (/^[a-z]+:/.test(target) || target.startsWith("#")) return whole;
    const [file, anchor] = target.split("#");
    // Resolve relative to the file it was written in.
    const parts = (fromDir ? fromDir + "/" : "") + file;
    const normal = parts
      .split("/")
      .reduce((acc, seg) => {
        if (seg === "..") acc.pop();
        else if (seg !== "." && seg !== "") acc.push(seg);
        return acc;
      }, [])
      .join("/");
    const slug = slugFor.get(normal);
    const suffix = anchor ? `#${anchor}` : "";
    if (slug) return `](/${slug}/${suffix})`;
    return `](${github}${normal}${suffix})`;
  });
}

function split(markdown) {
  // The first H1 becomes the page title; Starlight renders its own.
  const match = markdown.match(/^# (.+)\n/m);
  const title = match ? match[1].trim() : "LDF";
  const body = match ? markdown.replace(match[0], "") : markdown;
  return { title, body: body.replace(/^\n+/, "") };
}

await mkdir(out, { recursive: true });
for (const [path, slug, description, override] of pages) {
  const source = await readFile(join(repo, path), "utf8");
  const { title, body } = split(source);
  const page = [
    "---",
    `title: ${JSON.stringify(override ?? title)}`,
    `description: ${JSON.stringify(description)}`,
    "---",
    "",
    rewriteLinks(body, path),
  ].join("\n");
  await mkdir(dirname(join(out, `${slug}.md`)), { recursive: true });
  await writeFile(join(out, `${slug}.md`), page);
  console.log(`${path} -> ${slug}.md`);
}

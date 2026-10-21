// Mirrors the repository-root Markdown into the site's content collection.
//
// The documentation pages (spec, guides, design, research, roadmap) live in
// src/content/docs/docs and are edited there. AGENTS.md, CHANGELOG.md and
// CONTRIBUTING.md stay at the repository root, where GitHub, crates.io and
// coding agents look for them, and so does the baseline report under
// reports/; this script copies those into src/content/docs/docs with the
// frontmatter Starlight needs and links rewritten to site paths. The copies
// are generated and gitignored. Runs under bun with nothing but the standard
// library.
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
  ["reports/2026-10-17-baseline/README.md", "docs/results/baseline", "Perf, tokens and comprehension runs on an M2 Pro, with the raw files next to them.", "Baseline report (October 2026)"],
  ["reports/2026-10-19-stage-c/README.md", "docs/results/stage-c", "Claude Opus 5.5 on held-out data: LDF v0.1, threshold 0.75 and hoisted arrays against JSON.", "Stage C: Opus 5.5 on held-out data"],
  ["reports/2026-10-20-hook/README.md", "docs/results/hook", "The Claude Code Bash hook on the medium fixtures: exact o200k tokens per dataset and model, per-call timing.", "Bash hook in Claude Code"],
  ["AGENTS.md", "docs/working-notes", "Architecture, commands and conventions for anyone touching the repository.", "Working notes"],
  ["CHANGELOG.md", "docs/changelog", "What changed, by version of the specification."],
  ["CONTRIBUTING.md", "docs/contributing", "The bar for a change: measurements first, fixtures with the spec, pinned dependencies."],
];

// Pages that live in the site already, so links to their files from the
// mirrored Markdown land on the page.
const sitePages = ["spec", "agents", "mcp", "methodology", "design", "research", "roadmap"].map((name) => [
  `site/src/content/docs/docs/${name}.md`,
  `docs/${name}`,
]);

// The README is not mirrored: the hand-written overview under /docs/
// replaces it on the site, so links to it land there.
const slugFor = new Map([...pages.map(([path, slug]) => [path, slug]), ...sitePages, ["README.md", "docs"]]);

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

// The selector's evidence and its verdicts, for the chart in the docs
// (src/components/EvidenceChart.astro) and the landing grid (VerdictGrid.astro).
const data = join(here, "..", "src", "data");
await mkdir(data, { recursive: true });
await writeFile(join(data, "evidence.json"), await readFile(join(repo, "reports/evidence/evidence.json")));
console.log("reports/evidence/evidence.json -> src/data/evidence.json");
await writeFile(join(data, "verdicts.json"), await readFile(join(repo, "reports/evidence/verdicts.json")));
console.log("reports/evidence/verdicts.json -> src/data/verdicts.json");

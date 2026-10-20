// ldflabs.org. The landing page is src/pages/index.astro; everything under
// /docs/ is Starlight. The overview and the five-minute guide are
// hand-written MDX in src/content/docs/docs; the specification, the
// ldf-mcp guide, methodology, design and research notes, the measured
// report, the roadmap and the working notes come from the repository's
// Markdown, mirrored by scripts/sync-docs.mjs before every build. Edit
// those files, not the copies.
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

export default defineConfig({
  site: "https://ldflabs.org",
  redirects: {
    "/spec": "/docs/spec/",
    "/mcp": "/docs/mcp/",
  },
  integrations: [
    starlight({
      title: "LDF",
      description:
        "Language Data Format: a lossless, line-oriented text encoding of JSON records for language models, measured against JSON model by model.",
      logo: { src: "./src/assets/logo.svg", replacesTitle: false },
      favicon: "/favicon.svg",
      // Page views, cookieless, on the self-hosted Umami. The landing page
      // carries the same tag in src/layouts/Landing.astro.
      head: [
        {
          tag: "script",
          attrs: {
            defer: true,
            src: "https://a.caioricciuti.com/b.js",
            "data-website-id": "1a84e1c7-763c-4551-8bc7-aaa3b6e79eae",
          },
        },
      ],
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/caioricciuti/ldf" },
      ],
      components: {
        SocialIcons: "./src/components/docs/HeaderLinks.astro",
      },
      customCss: ["./src/styles/custom.css"],
      expressiveCode: {
        themes: ["github-dark-dimmed", "github-light"],
        styleOverrides: {
          borderRadius: "6px",
          borderColor: "var(--ldf-frame)",
          codeFontFamily: "var(--sl-font-mono)",
          codeFontSize: "0.8125rem",
          codeLineHeight: "1.6",
          codePaddingBlock: "0.75rem",
          codePaddingInline: "1rem",
          codeBackground: "var(--ldf-surface)",
          uiFontFamily: "var(--sl-font)",
          frames: {
            shadowColor: "transparent",
          },
        },
        // No terminal title bar on shell blocks: three dots and an empty
        // bar cost a line of height on every command.
        defaultProps: { frame: "none" },
      },
      sidebar: [
        {
          label: "Start here",
          items: [
            { label: "Overview", slug: "docs" },
            { label: "The format in five minutes", slug: "docs/format" },
            { label: "ldf-mcp: LDF between a source and the model", slug: "docs/mcp" },
            { label: "With Claude Code, Codex and OpenCode", slug: "docs/agents" },
          ],
        },
        {
          label: "Reference",
          items: [
            { label: "Specification (v0.2)", slug: "docs/spec" },
            { label: "Benchmark methodology", slug: "docs/methodology" },
            { label: "Design candidates", slug: "docs/design" },
            { label: "Research notes", slug: "docs/research" },
          ],
        },
        {
          label: "Results",
          items: [
            { label: "Bash hook in Claude Code (2026-10-20)", slug: "docs/results/hook" },
            { label: "Stage C: Opus 5.5, held-out (2026-10-19)", slug: "docs/results/stage-c" },
            { label: "Baseline report (2026-10-17)", slug: "docs/results/baseline" },
          ],
        },
        {
          label: "Project",
          items: [
            { label: "Roadmap", slug: "docs/roadmap" },
            { label: "Changelog", slug: "docs/changelog" },
            { label: "Contributing", slug: "docs/contributing" },
            { label: "Working notes", slug: "docs/working-notes" },
          ],
        },
      ],
      lastUpdated: false,
      pagination: true,
    }),
  ],
});

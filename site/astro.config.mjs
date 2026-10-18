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
        "Language Data Format: a lossless, line-oriented text encoding of JSON records that language models read for 35 to 45 percent fewer tokens than JSON.",
      logo: { src: "./src/assets/logo.svg", replacesTitle: false },
      favicon: "/favicon.svg",
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
          borderRadius: "10px",
          borderColor: "var(--ldf-frame)",
          codeFontFamily: "var(--sl-font-mono)",
          uiFontFamily: "var(--sl-font)",
          frames: {
            shadowColor: "transparent",
          },
        },
      },
      sidebar: [
        {
          label: "Start here",
          items: [
            { label: "Overview", slug: "docs" },
            { label: "The format in five minutes", slug: "docs/format" },
            { label: "ldf-mcp: LDF between a source and the model", slug: "docs/mcp" },
          ],
        },
        {
          label: "Reference",
          items: [
            { label: "Specification (v0.1)", slug: "docs/spec" },
            { label: "Benchmark methodology", slug: "docs/methodology" },
            { label: "Design candidates", slug: "docs/design" },
            { label: "Research notes", slug: "docs/research" },
          ],
        },
        {
          label: "Results",
          items: [{ label: "Baseline report (October 2026)", slug: "docs/results/baseline" }],
        },
        {
          label: "Project",
          items: [
            { label: "Roadmap", slug: "docs/roadmap" },
            { label: "Working notes", slug: "docs/working-notes" },
          ],
        },
      ],
      lastUpdated: false,
      pagination: true,
    }),
  ],
});

// @ts-expect-error Vitest runs this regression check in Node.
import { readFileSync } from "node:fs";
// @ts-expect-error Vitest runs this regression check in Node.
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

describe("dashboard card layout", () => {
  it("lets quota cards grow with their chart and status content", () => {
    const styles = readFileSync(resolve("src/styles.css"), "utf8");
    const quotaCardRule = styles.match(/\.quota-card\s*\{([^}]*)\}/)?.[1];

    expect(quotaCardRule).toBeDefined();
    expect(quotaCardRule).not.toMatch(/(?:^|\n)\s*height:\s*100%;/);
  });

  it("keeps the content grid item constrained so the inner view can scroll", () => {
    const styles = readFileSync(resolve("src/styles.css"), "utf8");
    const contentRule = styles.match(/\.content\s*\{([^}]*)\}/)?.[1];
    const scrollerRule = styles.match(/\.content-scroll\s*\{([^}]*)\}/)?.[1];
    const bodyRule = styles.match(/\.content-scroll-body\s*\{([^}]*)\}/)?.[1];
    const trackRule = styles.match(
      /\.content-scroll::-webkit-scrollbar-track\s*\{([^}]*)\}/,
    )?.[1];
    const thumbHoverRule = styles.match(
      /\.content-scroll::-webkit-scrollbar-thumb:hover,\s*\n\.content-scroll::-webkit-scrollbar-thumb:active\s*\{([^}]*)\}/,
    )?.[1];

    expect(contentRule).toMatch(/min-height:\s*0;/);
    expect(contentRule).toMatch(/height:\s*100%;/);
    expect(scrollerRule).toMatch(/min-height:\s*0;/);
    expect(scrollerRule).toMatch(/overflow:\s*auto;/);
    expect(scrollerRule).toMatch(/margin-right:\s*calc\(-1 \* var\(--window-inset\)\);/);
    expect(scrollerRule).toMatch(/mask-image:/);
    expect(scrollerRule).toMatch(/mask-size:\s*calc\(100% - 10px\) 100%, 10px 100%;/);
    expect(bodyRule).not.toMatch(/mask-image:/);
    expect(trackRule).toMatch(/margin-top:\s*52px;/);
    expect(thumbHoverRule).not.toMatch(/transform:\s*scale/);
    expect(thumbHoverRule).toMatch(/width:\s*8px;/);
    expect(styles).toMatch(/--scrollbar-thumb:\s*#9b9b9b;/);
  });

  it("uses native non-selectable text while keeping form controls editable", () => {
    const styles = readFileSync(resolve("src/styles.css"), "utf8");
    const bodyRule = styles.match(/body\s*\{([^}]*)\}/)?.[1];
    const buttonRule = styles.match(/button\s*\{([^}]*)\}/)?.[1];

    expect(bodyRule).toMatch(/user-select:\s*none;/);
    expect(buttonRule).toMatch(/cursor:\s*default;/);
    expect(buttonRule).toMatch(/user-select:\s*none;/);
    expect(styles).toMatch(/input,\s*\ntextarea,\s*\nselect\s*\{[^}]*user-select:\s*text;/);
  });

  it("keeps the glass toolbar top-aligned while centering the title to it", () => {
    const styles = readFileSync(resolve("src/styles.css"), "utf8");
    const topbarRule = styles.match(/\.topbar\s*\{([^}]*)\}/)?.[1];
    const titleRule = styles.match(/\.topbar-title\s*\{([^}]*)\}/)?.[1];
    const controlsRule = styles.match(/\.topbar-actions\s*\{([^}]*)\}/)?.[1];

    expect(topbarRule).toMatch(/top:\s*0;/);
    expect(topbarRule).toMatch(/align-items:\s*flex-start;/);
    expect(topbarRule).toMatch(/min-height:\s*52px;/);
    expect(topbarRule).toMatch(/padding:\s*0 10px 0 8px;/);
    expect(controlsRule).toMatch(/min-height:\s*42px;/);
    expect(controlsRule).toMatch(/margin-left:\s*-10px;/);
    expect(titleRule).toMatch(/min-height:\s*42px;/);
    expect(titleRule).toMatch(/align-items:\s*center;/);
  });
});

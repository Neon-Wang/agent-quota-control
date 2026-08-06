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
});

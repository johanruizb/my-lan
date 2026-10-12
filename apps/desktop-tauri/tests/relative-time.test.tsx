import { render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { RelativeTime } from "@/components/relative-time";

describe("RelativeTime", () => {
    // Set a fixed system time for deterministic relative time assertions
    const NOW = new Date("2025-01-15T12:00:00.000Z");

    beforeEach(() => {
        vi.useFakeTimers();
        vi.setSystemTime(NOW);
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe("Valid date input types", () => {
        it("renders ISO date string with relative text and absolute timestamp title", () => {
            const dateStr = "2025-01-15T10:00:00.000Z";
            const { container } = render(<RelativeTime value={dateStr} />);

            const span = container.querySelector("span");
            expect(span).not.toBeNull();
            expect(span?.textContent).toMatch(/hace 2 horas/i);
            expect(span?.getAttribute("title")).toBeTruthy();
            expect(span?.getAttribute("title")).not.toBe("—");
        });

        it("renders Date object correctly", () => {
            const dateObj = new Date("2025-01-15T11:00:00.000Z");
            const { container } = render(<RelativeTime value={dateObj} />);

            const span = container.querySelector("span");
            expect(span?.textContent).toMatch(/hace una hora|hace 1 hora/i);
            expect(span?.getAttribute("title")).toBeTruthy();
            expect(span?.getAttribute("title")).not.toBe("—");
        });

        it("renders timestamp number (milliseconds) correctly", () => {
            const timestamp = new Date("2025-01-15T09:00:00.000Z").getTime();
            const { container } = render(<RelativeTime value={timestamp} />);

            const span = container.querySelector("span");
            expect(span?.textContent).toMatch(/hace 3 horas/i);
            expect(span?.getAttribute("title")).toBeTruthy();
            expect(span?.getAttribute("title")).not.toBe("—");
        });
    });

    describe("Edge cases and invalid inputs", () => {
        it("renders fallback '—' and title '—' when value is null", () => {
            const { container } = render(<RelativeTime value={null} />);
            const span = container.querySelector("span");
            expect(span?.textContent).toBe("—");
            expect(span?.getAttribute("title")).toBe("—");
        });

        it("renders fallback '—' and title '—' when value is undefined", () => {
            const { container } = render(<RelativeTime value={undefined} />);
            const span = container.querySelector("span");
            expect(span?.textContent).toBe("—");
            expect(span?.getAttribute("title")).toBe("—");
        });

        it("renders fallback '—' and title '—' when value is an invalid date string", () => {
            const { container } = render(<RelativeTime value="invalid-date-string" />);
            const span = container.querySelector("span");
            expect(span?.textContent).toBe("—");
            expect(span?.getAttribute("title")).toBe("—");
        });

        it("renders fallback '—' and title '—' when value is NaN", () => {
            const { container } = render(<RelativeTime value={NaN} />);
            const span = container.querySelector("span");
            expect(span?.textContent).toBe("—");
            expect(span?.getAttribute("title")).toBe("—");
        });

        it("renders fallback '—' and title '—' when value is an empty string", () => {
            const { container } = render(<RelativeTime value="" />);
            const span = container.querySelector("span");
            expect(span?.textContent).toBe("—");
            expect(span?.getAttribute("title")).toBe("—");
        });
    });

    describe("Styling and class names", () => {
        it("applies default class 'tabular-nums'", () => {
            const { container } = render(<RelativeTime value="2025-01-15T10:00:00.000Z" />);
            const span = container.querySelector("span");
            expect(span?.className).toContain("tabular-nums");
        });

        it("merges custom className with 'tabular-nums'", () => {
            const { container } = render(
                <RelativeTime value="2025-01-15T10:00:00.000Z" className="text-muted-foreground text-xs" />
            );
            const span = container.querySelector("span");
            expect(span?.className).toContain("tabular-nums");
            expect(span?.className).toContain("text-muted-foreground");
            expect(span?.className).toContain("text-xs");
        });
    });
});

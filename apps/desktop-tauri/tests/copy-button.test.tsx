import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { CopyButton } from "@/components/copy-button";

const toastMock = vi.fn();
vi.mock("@/components/ui/toast", () => ({
    useToast: () => ({ toast: toastMock }),
}));

describe("CopyButton component", () => {
    beforeEach(() => {
        toastMock.mockReset();
        vi.restoreAllMocks();
    });

    it("no renderiza nada si el valor es null, undefined, vacío o '—'", () => {
        const { container: c1 } = render(<CopyButton value={null} label="IP" />);
        expect(c1).toBeEmptyDOMElement();

        const { container: c2 } = render(<CopyButton value={undefined} label="IP" />);
        expect(c2).toBeEmptyDOMElement();

        const { container: c3 } = render(<CopyButton value="" label="IP" />);
        expect(c3).toBeEmptyDOMElement();

        const { container: c4 } = render(<CopyButton value="—" label="IP" />);
        expect(c4).toBeEmptyDOMElement();
    });

    it("renderiza el botón con accesibilidad y aria-label", () => {
        render(<CopyButton value="192.168.1.1" label="IP" />);
        const button = screen.getByRole("button", { name: "Copiar IP" });
        expect(button).toBeInTheDocument();
        expect(button).toHaveAttribute("title", "Copiar IP");
    });

    it("copia al portapapeles usando navigator.clipboard.writeText y muestra toast", async () => {
        const writeTextMock = vi.fn().mockResolvedValue(undefined);
        Object.assign(navigator, {
            clipboard: {
                writeText: writeTextMock,
            },
        });

        render(<CopyButton value="192.168.1.42" label="IP" />);
        const button = screen.getByRole("button", { name: "Copiar IP" });

        fireEvent.click(button);

        await waitFor(() => {
            expect(writeTextMock).toHaveBeenCalledWith("192.168.1.42");
        });
        expect(toastMock).toHaveBeenCalledWith(
            "IP copiado al portapapeles.",
            "default",
        );
    });

    it("usa fallback de execCommand cuando navigator.clipboard.writeText falla", async () => {
        const writeTextMock = vi.fn().mockRejectedValue(new Error("Permission denied"));
        Object.assign(navigator, {
            clipboard: {
                writeText: writeTextMock,
            },
        });

        const execCommandMock = vi.fn().mockReturnValue(true);
        document.execCommand = execCommandMock;

        render(<CopyButton value="aa:bb:cc:dd:ee:ff" label="MAC" />);
        const button = screen.getByRole("button", { name: "Copiar MAC" });

        fireEvent.click(button);

        await waitFor(() => {
            expect(execCommandMock).toHaveBeenCalledWith("copy");
        });
        expect(toastMock).toHaveBeenCalledWith(
            "MAC copiado al portapapeles.",
            "default",
        );
    });

    it("muestra toast de error cuando ambos métodos de copiado fallan", async () => {
        const writeTextMock = vi.fn().mockRejectedValue(new Error("Permission denied"));
        Object.assign(navigator, {
            clipboard: {
                writeText: writeTextMock,
            },
        });

        const execCommandMock = vi.fn().mockReturnValue(false);
        document.execCommand = execCommandMock;

        render(<CopyButton value="test-banner" label="Banner" />);
        const button = screen.getByRole("button", { name: "Copiar Banner" });

        fireEvent.click(button);

        await waitFor(() => {
            expect(toastMock).toHaveBeenCalledWith(
                "No se pudo copiar banner.",
                "error",
            );
        });
    });
});

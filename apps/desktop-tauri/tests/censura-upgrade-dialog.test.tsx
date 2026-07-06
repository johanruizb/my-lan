// AC-18: Tests de `censura-upgrade-dialog.tsx` (mock onCensorshipFresh).
// Distingue install nuevo (evento `censorship:fresh` suprime el dialog) de
// upgrade (timeout FRESH_WAIT_MS lo muestra). Usa fake timers para el race
// listener/timeout y verifica la interacción de los botones.

import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { CensuraUpgradeDialog } from "@/components/censura-upgrade-dialog";

const SHOWN_KEY = "censorship_dialog_shown";

// `vi.hoisted` garantiza que los mocks existan cuando el factory de `vi.mock`
// se ejecute (los factories se hoistan antes que cualquier `const` del módulo).
const { onCensorshipFreshMock, setCensorshipEnabledMock } = vi.hoisted(() => ({
    onCensorshipFreshMock: vi.fn(),
    setCensorshipEnabledMock: vi.fn(),
}));

vi.mock("@/lib/tauri", () => ({
    onCensorshipFresh: (cb: () => void) => onCensorshipFreshMock(cb),
}));

vi.mock("@/components/censorship-provider", () => ({
    useCensorship: () => ({
        censorshipEnabled: true,
        setCensorshipEnabled: setCensorshipEnabledMock,
        toggle: vi.fn(),
    }),
}));

function freshHandlerCapture(cb: () => void) {
    // Captura el callback para invocarlo manualmente (simula el evento).
    (freshHandlerCapture as unknown as { _cb?: () => void })._cb = cb;
    return Promise.resolve(() => {});
}

function invokeCapturedFresh() {
    const cb = (freshHandlerCapture as unknown as { _cb?: () => void })._cb;
    if (cb) cb();
}

beforeEach(() => {
    localStorage.clear();
    onCensorshipFreshMock.mockReset();
    setCensorshipEnabledMock.mockReset();
    onCensorshipFreshMock.mockImplementation(freshHandlerCapture);
    vi.useFakeTimers();
});

afterEach(() => {
    vi.useRealTimers();
});

describe("CensuraUpgradeDialog — fresh-install vs upgrade (AC-18)", () => {
    it("ya mostrado: no suscribe listener ni renderiza nada", () => {
        localStorage.setItem(SHOWN_KEY, "1");
        render(<CensuraUpgradeDialog />);
        expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
        expect(onCensorshipFreshMock).not.toHaveBeenCalled();
    });

    it("fresh-install: el evento censorship:fresh suprime el dialog y marca SHOWN", async () => {
        render(<CensuraUpgradeDialog />);
        // El evento fresh llega antes del timeout: invocamos el callback.
        await act(async () => {
            invokeCapturedFresh();
        });
        expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
        expect(localStorage.getItem(SHOWN_KEY)).toBe("1");
    });

    it("upgrade: el timeout muestra el dialog una sola vez", async () => {
        // onCensorshipFresh captura el cb pero NUNCA lo invoca (upgrade).
        render(<CensuraUpgradeDialog />);
        // flush de la microtask del .then(onCensorshipFresh)
        await act(async () => {
            await Promise.resolve();
        });
        // El evento no llega → timeout asume upgrade.
        act(() => {
            vi.advanceTimersByTime(2000);
        });
        expect(screen.getByRole("dialog")).toBeInTheDocument();
        expect(
            screen.getByText("Modo censura"),
        ).toBeInTheDocument();
        expect(
            screen.getByRole("button", { name: /Mantener visible/ }),
        ).toBeInTheDocument();
        expect(
            screen.getByRole("button", { name: /Activar \(recomendado\)/ }),
        ).toBeInTheDocument();
    });

    it("upgrade: click 'Activar (recomendado)' persiste censura ON y cierra", async () => {
        render(<CensuraUpgradeDialog />);
        await act(async () => {
            await Promise.resolve();
        });
        act(() => {
            vi.advanceTimersByTime(2000);
        });
        const activate = screen.getByRole("button", {
            name: /Activar \(recomendado\)/,
        });
        fireEvent.click(activate);
        expect(setCensorshipEnabledMock).toHaveBeenCalledWith(true);
        expect(localStorage.getItem(SHOWN_KEY)).toBe("1");
        expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    it("upgrade: click 'Mantener visible' persiste censura OFF y cierra", async () => {
        render(<CensuraUpgradeDialog />);
        await act(async () => {
            await Promise.resolve();
        });
        act(() => {
            vi.advanceTimersByTime(2000);
        });
        const keep = screen.getByRole("button", { name: /Mantener visible/ });
        fireEvent.click(keep);
        expect(setCensorshipEnabledMock).toHaveBeenCalledWith(false);
        expect(localStorage.getItem(SHOWN_KEY)).toBe("1");
        expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });
});
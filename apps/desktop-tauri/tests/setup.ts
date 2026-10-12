/// <reference types="vitest/globals" />
import "@testing-library/jest-dom/vitest";

// Mitiga act() warning (React 18.3 + RTL 16.3 + Vitest jsdom, issue #1413).
globalThis.IS_REACT_ACT_ENVIRONMENT = true;

// Mitiga `window.__TAURI_INTERNALS__` undefined en jsdom (Tauri issue #14281):
// algunos módulos de `@tauri-apps/api` acceden a `__TAURI_INTERNALS__` al
// importarse; sin el stub, los tests de componentes que tocan IPC fallan al
// render. `invoke` y `transformCallback` quedan como no-ops controlados por
// los mocks de cada suite.
vi.stubGlobal("__TAURI_INTERNALS__", {
    invoke: vi.fn(),
    transformCallback: vi.fn(),
});

// Mitiga discordancia entre jsdom AbortSignal y Node undici Request (Node 22/24 + jsdom + react-router v7):
// jsdom sobrescribe `globalThis.AbortSignal` con su propia clase, pero no implementa `Request`,
// por lo que `Request` proviene del runtime de Node (undici). Al navegar en react-router,
// este crea un Request con el AbortSignal de jsdom, provocando `TypeError: RequestInit: Expected
// signal ("AbortSignal {}") to be an instance of AbortSignal`.
if (typeof globalThis.Request !== "undefined") {
    const OriginalRequest = globalThis.Request;
    try {
        class PatchedRequest extends OriginalRequest {
            constructor(input: RequestInfo | URL, init?: RequestInit) {
                if (init && "signal" in init) {
                    const { signal, ...rest } = init;
                    super(input, rest);
                } else {
                    super(input, init);
                }
            }
        }
        globalThis.Request = PatchedRequest as any;
    } catch {
        // En entornos sin Request nativo o si falla el patch, mantener OriginalRequest
    }
}

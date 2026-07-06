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
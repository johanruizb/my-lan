// AC-13: Tests unit del helper `deriveTrustState` (`lib/trust-state.ts`).
// Cubre las 3 ramas (trusted/recognized/unknown) + caso NaN
// (`confidence: "high"` no numérico del backend legacy).
// Sin dependencias de React ni IPC.

import { deriveTrustState } from "@/lib/trust-state";

describe("deriveTrustState", () => {
    it("retorna 'trusted' cuando is_trusted es true (independiente de confidence)", () => {
        expect(deriveTrustState({ is_trusted: true, confidence: "90" })).toBe(
            "trusted",
        );
        expect(deriveTrustState({ is_trusted: true, confidence: "0" })).toBe(
            "trusted",
        );
    });

    it("retorna 'recognized' cuando is_trusted false y confidence >= 50", () => {
        expect(deriveTrustState({ is_trusted: false, confidence: "70" })).toBe(
            "recognized",
        );
        expect(deriveTrustState({ is_trusted: false, confidence: "50" })).toBe(
            "recognized",
        );
    });

    it("retorna 'unknown' cuando is_trusted false y confidence < 50", () => {
        expect(deriveTrustState({ is_trusted: false, confidence: "30" })).toBe(
            "unknown",
        );
        expect(deriveTrustState({ is_trusted: false, confidence: "0" })).toBe(
            "unknown",
        );
    });

    it("retorna 'unknown' cuando confidence no es numérico (NaN < 50)", () => {
        expect(deriveTrustState({ is_trusted: false, confidence: "high" })).toBe(
            "unknown",
        );
    });
});
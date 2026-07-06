// AC-20: Tests de hook `useScan` con renderHook.
// `useScan` consume `ScanContext`; la lógica de suscripción a eventos
// (onScanDevice/onScanCancelled) vive en `AppInner` y propaga los cambios al
// contexto. Aquí se valida que el hook refleja el valor del provider y que
// `deviceKey`/`mergeDevice` implementan la identidad de merge (devicesFound
// crece sin duplicados) y el estado de cancelación (scanning=false).

import { renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ScanContext, deviceKey, mergeDevice, useScan } from "@/App";
import { makeDevice } from "./fixtures";

function providerValue(overrides: Partial<{
    scanning: boolean;
    scanId: string | null;
    devicesFound: ReturnType<typeof makeDevice>[];
    progress: null;
    startScan: () => void;
    cancel: () => void;
}> = {}) {
    return {
        scanning: false,
        scanId: null,
        progress: null,
        devicesFound: [],
        startScan: vi.fn(),
        cancel: vi.fn(),
        ...overrides,
    };
}

function wrapperWith(value: ReturnType<typeof providerValue>) {
    return function Wrapper({ children }: { children: React.ReactNode }) {
        return (
            <ScanContext.Provider value={value}>
                {children}
            </ScanContext.Provider>
        );
    };
}

describe("useScan — hook de contexto (AC-20)", () => {
    // Silenciar el stack trace de react-dom que React 18 loguea al throw del
    // hook fuera del provider (code-review MINOR: limpiar output CI).
    beforeEach(() => {
        vi.spyOn(console, "error").mockImplementation(() => {});
    });
    afterEach(() => {
        vi.restoreAllMocks();
    });

    it("throw cuando se usa fuera de ScanProvider", () => {
        expect(() => renderHook(() => useScan())).toThrow(
            /useScan debe usarse dentro de ScanProvider/,
        );
    });

    it("retorna el valor del ScanContext.Provider", () => {
        const value = providerValue({ scanId: "scan-42" });
        const { result } = renderHook(() => useScan(), {
            wrapper: wrapperWith(value),
        });
        expect(result.current.scanId).toBe("scan-42");
        expect(result.current.scanning).toBe(false);
        expect(result.current.devicesFound).toEqual([]);
    });
});

describe("useScan — devicesFound crece y scanning=false al cancel (AC-20)", () => {
    let value: ReturnType<typeof providerValue>;

    beforeEach(() => {
        value = providerValue();
    });

    it("devicesFound crece al recibir onScanDevice (via contexto)", () => {
        const d1 = makeDevice({ id: "dev-1" });
        const d2 = makeDevice({ id: "dev-2", primary_ip: "192.168.1.11" });

        // Primer evento: 1 device
        const { result, rerender } = renderHook(() => useScan(), {
            wrapper: wrapperWith(value),
        });
        expect(result.current.devicesFound).toHaveLength(0);

        // AppInner recibió onScanDevice → devicesFound crece
        value.devicesFound = [d1];
        rerender({ wrapper: wrapperWith(value) });
        expect(result.current.devicesFound).toHaveLength(1);

        // Segundo evento: 2 devices
        value.devicesFound = [d1, d2];
        rerender({ wrapper: wrapperWith(value) });
        expect(result.current.devicesFound).toHaveLength(2);
    });

    it("scanning pasa a false al cancelar (via contexto)", () => {
        value.scanning = true;
        const { result, rerender } = renderHook(() => useScan(), {
            wrapper: wrapperWith(value),
        });
        expect(result.current.scanning).toBe(true);

        // AppInner recibió onScanCancelled → scanning=false
        value.scanning = false;
        rerender({ wrapper: wrapperWith(value) });
        expect(result.current.scanning).toBe(false);
    });
});

describe("deviceKey — identidad de merge (AC-20)", () => {
    it("prioriza primary_ip", () => {
        expect(
            deviceKey(
                makeDevice({ primary_ip: "192.168.1.10", primary_mac: "aa" }),
            ),
        ).toBe("192.168.1.10");
    });

    it("fallback a primary_mac si primary_ip es null", () => {
        expect(
            deviceKey(makeDevice({ primary_ip: null, primary_mac: "aa:bb" })),
        ).toBe("aa:bb");
    });

    it("fallback a id si ambos son null", () => {
        expect(
            deviceKey(
                makeDevice({ id: "dev-x", primary_ip: null, primary_mac: null }),
            ),
        ).toBe("dev-x");
    });
});

describe("mergeDevice — dedup por identidad (AC-20, code-review MAJOR #2)", () => {
    it("añade un device nuevo a la lista vacía", () => {
        const d = makeDevice({ id: "dev-1", primary_ip: "192.168.1.10" });
        expect(mergeDevice([], d)).toEqual([d]);
    });

    it("reemplaza el device existente por primary_ip (dedup)", () => {
        const d1 = makeDevice({ id: "dev-1", primary_ip: "192.168.1.10", display_name: "viejo" });
        const d2 = makeDevice({ id: "dev-1", primary_ip: "192.168.1.10", display_name: "nuevo" });
        const result = mergeDevice([d1], d2);
        expect(result).toHaveLength(1);
        expect(result[0].display_name).toBe("nuevo");
    });

    it("añade un device con ip distinta sin reemplazar", () => {
        const d1 = makeDevice({ id: "dev-1", primary_ip: "192.168.1.10" });
        const d2 = makeDevice({ id: "dev-2", primary_ip: "192.168.1.11" });
        const result = mergeDevice([d1], d2);
        expect(result).toHaveLength(2);
    });

    it("dedup por primary_mac cuando primary_ip es null", () => {
        const d1 = makeDevice({ id: "dev-1", primary_ip: null, primary_mac: "aa:bb" });
        const d2 = makeDevice({ id: "dev-1", primary_ip: null, primary_mac: "aa:bb", display_name: "nuevo" });
        const result = mergeDevice([d1], d2);
        expect(result).toHaveLength(1);
        expect(result[0].display_name).toBe("nuevo");
    });
});
// AC-16: Tests de componente puro `masked-value.tsx` con RTL.
// Cubre censura off/on, MAC (placeholder constante, no revelable), IP
// (blur + reveal-on-hover) y campo no sensible (passthrough).
//
// Se mockea `@/components/censorship-provider` para controlar
// `censorshipEnabled` sin montar el provider real.

import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useCensorship } from "@/components/censorship-provider";
import { MaskedValue } from "@/components/masked-value";

vi.mock("@/components/censorship-provider", () => ({
    useCensorship: vi.fn(),
}));

function setEnabled(enabled: boolean) {
    vi.mocked(useCensorship).mockReturnValue({
        censorshipEnabled: enabled,
        setCensorshipEnabled: vi.fn(),
        toggle: vi.fn(),
    });
}

describe("MaskedValue — censura OFF", () => {
    beforeEach(() => setEnabled(false));

    it("renderiza el valor crudo para un campo IP", () => {
        render(<MaskedValue field="primary_ip" value="192.168.1.42" />);
        expect(screen.getByText("192.168.1.42")).toBeInTheDocument();
    });

    it("renderiza '—' cuando el valor es null", () => {
        render(<MaskedValue field="primary_ip" value={null} />);
        expect(screen.getByText("—")).toBeInTheDocument();
    });

    it("pasa el valor sin cambios para un campo no sensible", () => {
        render(<MaskedValue field="vendor" value="ExampleVendor" />);
        expect(screen.getByText("ExampleVendor")).toBeInTheDocument();
    });
});

describe("MaskedValue — censura ON", () => {
    beforeEach(() => setEnabled(true));

    it("MAC: placeholder constante '••••', no revelable", () => {
        render(<MaskedValue field="primary_mac" value="aa:bb:cc:dd:ee:ff" />);
        expect(screen.getByText("••••")).toBeInTheDocument();
    });

    it("IP: muestra el valor enmascarado por defecto", () => {
        render(<MaskedValue field="primary_ip" value="192.168.1.42" />);
        expect(screen.getByText("192.168.*.*")).toBeInTheDocument();
    });

    it("IP: revela el valor real al mouseEnter y re-enmascara al mouseLeave", () => {
        render(<MaskedValue field="primary_ip" value="192.168.1.42" />);
        const span = screen.getByText("192.168.*.*");
        fireEvent.mouseEnter(span);
        expect(screen.getByText("192.168.1.42")).toBeInTheDocument();
        fireEvent.mouseLeave(span);
        expect(screen.getByText("192.168.*.*")).toBeInTheDocument();
    });

    it("IP: revela el valor real al focus y re-enmascara al blur", () => {
        render(<MaskedValue field="primary_ip" value="10.0.0.1" />);
        const span = screen.getByText("10.0.*.*");
        fireEvent.focus(span);
        expect(screen.getByText("10.0.0.1")).toBeInTheDocument();
        fireEvent.blur(span);
        expect(screen.getByText("10.0.*.*")).toBeInTheDocument();
    });

    it("campo no sensible: passthrough del valor crudo", () => {
        render(<MaskedValue field="vendor" value="ExampleVendor" />);
        expect(screen.getByText("ExampleVendor")).toBeInTheDocument();
    });

    it("hostname: enmascarado como maskHostname", () => {
        render(<MaskedValue field="hostname" value="router.lan" />);
        expect(screen.getByText("router.*")).toBeInTheDocument();
    });
});
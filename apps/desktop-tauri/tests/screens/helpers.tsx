// Helper compartido para tests de screens (AC-22).
// Provee MemoryRouter + TooltipProvider y reexporta fixtures.
// Los `vi.mock` de módulos se declaran en cada archivo de test (Vitest los
// hoista por archivo; los de un helper importado no se aplican al test).

import { render } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { TooltipProvider } from "@/components/ui/tooltip";
import { deviceFixtures } from "../fixtures";

export function renderWithProviders(
    ui: React.ReactElement,
    initialRoutes: string[] = ["/"],
) {
    return render(
        <MemoryRouter initialEntries={initialRoutes}>
            <TooltipProvider delayDuration={0}>{ui}</TooltipProvider>
        </MemoryRouter>,
    );
}

export { deviceFixtures };
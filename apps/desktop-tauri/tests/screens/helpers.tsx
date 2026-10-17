// Helper compartido para tests de screens (AC-22).
// Provee un data router (createMemoryRouter) + TooltipProvider y reexporta
// fixtures. Los `vi.mock` de módulos se declaran en cada archivo de test
// (Vitest los hoista por archivo; los de un helper importado no se aplican
// al test).

import React from "react";
import { render } from "@testing-library/react";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { TooltipProvider } from "@/components/ui/tooltip";
import { deviceFixtures } from "../fixtures";

export function renderWithProviders(
    ui: React.ReactElement,
    initialRoutes: string[] = ["/"],
    options?: { strictMode?: boolean },
) {
    const content = options?.strictMode ? (
        <React.StrictMode>{ui}</React.StrictMode>
    ) : (
        ui
    );
    const router = createMemoryRouter(
        [
            {
                path: "*",
                element: (
                    <TooltipProvider delayDuration={0}>{content}</TooltipProvider>
                ),
            },
        ],
        { initialEntries: initialRoutes },
    );
    return render(<RouterProvider router={router} />);
}

export { deviceFixtures };

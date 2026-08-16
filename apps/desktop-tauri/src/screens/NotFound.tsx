import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/empty-state";
import { Compass } from "lucide-react";

// 404 (skill: strategic omission "No custom 404 page"). HashRouter sirve
// index.html para cualquier hash, así que sin catch-all el usuario ve un
// main vacío. Esta ruta muestra un EmptyState con vuelta al Dashboard.
export function NotFound() {
    return (
        <EmptyState
            icon={Compass}
            title="Página no encontrada"
            description="La ruta que buscas no existe o se movió."
            action={
                <Button asChild size="sm" className="gap-1.5">
                    <Link to="/">Volver al inicio</Link>
                </Button>
            }
        />
    );
}

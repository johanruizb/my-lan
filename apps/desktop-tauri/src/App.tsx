import { HashRouter, NavLink, Route, Routes } from "react-router-dom";
import { cn } from "@/lib/utils";
import { ToastProvider } from "@/components/ui/toast";
import { Dashboard } from "@/screens/Dashboard";
import { Devices } from "@/screens/Devices";
import { DeviceDetail } from "@/screens/DeviceDetail";
import { Scans } from "@/screens/Scans";
import { Settings } from "@/screens/Settings";

const navItems = [
  { to: "/", label: "Dashboard", end: true },
  { to: "/devices", label: "Dispositivos" },
  { to: "/scans", label: "Scans" },
  { to: "/settings", label: "Ajustes" },
];

function App() {
  return (
    <ToastProvider>
      <HashRouter>
        <main className="min-h-screen bg-background text-foreground">
          <div className="mx-auto flex max-w-5xl flex-col gap-6 px-6 py-8">
            <header className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
              <h1 className="text-2xl font-bold tracking-tight">MyLAN</h1>
              <nav className="flex flex-wrap gap-1">
                {navItems.map((n) => (
                  <NavLink
                    key={n.to}
                    to={n.to}
                    end={n.end}
                    className={({ isActive }) =>
                      cn(
                        "rounded-md px-3 py-1.5 text-sm font-medium transition-colors",
                        isActive
                          ? "bg-primary text-primary-foreground"
                          : "text-muted-foreground hover:bg-muted hover:text-foreground",
                      )
                    }
                  >
                    {n.label}
                  </NavLink>
                ))}
              </nav>
            </header>

            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/devices" element={<Devices />} />
              <Route path="/devices/:ip" element={<DeviceDetail />} />
              <Route path="/scans" element={<Scans />} />
              <Route path="/settings" element={<Settings />} />
            </Routes>
          </div>
        </main>
      </HashRouter>
    </ToastProvider>
  );
}

export default App;
import { createContext, useCallback, useContext, useState, type ReactNode } from "react";

type ToastVariant = "default" | "success" | "error";

interface Toast {
  id: number;
  message: string;
  variant: ToastVariant;
}

interface ToastContextValue {
  toast: (message: string, variant?: ToastVariant) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const toast = useCallback((message: string, variant: ToastVariant = "default") => {
    const id = Date.now() + Math.random();
    setToasts((t) => [...t, { id, message, variant }]);
    window.setTimeout(() => {
      setToasts((t) => t.filter((x) => x.id !== id));
    }, 4000);
  }, []);

  return (
    <ToastContext.Provider value={{ toast }}>
      {children}
      <div className="pointer-events-none fixed bottom-4 right-4 z-50 flex flex-col gap-2">
        {toasts.map((t) => (
          <div
            key={t.id}
            className={cnToast(t.variant)}
            role="status"
          >
            {t.message}
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}

function cnToast(variant: ToastVariant): string {
  const base =
    "pointer-events-auto rounded-md border px-4 py-2 text-sm shadow-md max-w-sm";
  const styles: Record<ToastVariant, string> = {
    default: "bg-card text-card-foreground border-border",
    success: "bg-green-50 text-green-900 border-green-300",
    error: "bg-red-50 text-red-900 border-red-300",
  };
  return `${base} ${styles[variant]}`;
}

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    throw new Error("useToast debe usarse dentro de ToastProvider");
  }
  return ctx;
}
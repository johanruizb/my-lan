import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useRef,
    useState,
    type ReactNode,
} from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getSettings } from "@/lib/tauri";
import { useToast } from "@/components/ui/toast";

export type UpdateStatus =
    | "idle"
    | "checking"
    | "available"
    | "downloading"
    | "ready"
    | "up-to-date"
    | "error";

export interface UpdateInfo {
    version: string;
    currentVersion: string;
    date?: string;
    body?: string;
}

export interface DownloadProgress {
    total?: number;
    downloaded: number;
    percent?: number;
}

export interface UpdaterContextValue {
    status: UpdateStatus;
    updateInfo: UpdateInfo | null;
    progress: DownloadProgress | null;
    error: string | null;
    checkForUpdates: (options?: { silent?: boolean }) => Promise<void>;
    downloadAndInstall: () => Promise<void>;
    restartApp: () => Promise<void>;
    dismissUpdate: () => void;
}

const UpdaterContext = createContext<UpdaterContextValue | null>(null);

export function useUpdater(): UpdaterContextValue {
    const ctx = useContext(UpdaterContext);
    if (!ctx) {
        throw new Error("useUpdater debe usarse dentro de UpdaterProvider");
    }
    return ctx;
}

export function UpdaterProvider({ children }: { children: ReactNode }) {
    const { toast } = useToast();
    const [status, setStatus] = useState<UpdateStatus>("idle");
    const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
    const [progress, setProgress] = useState<DownloadProgress | null>(null);
    const [error, setError] = useState<string | null>(null);

    const activeUpdateRef = useRef<Update | null>(null);
    const hasAutoCheckedRef = useRef(false);

    // Cierra el recurso de actualización activo al desmontar o reemplazar
    const closeActiveUpdate = useCallback(async () => {
        if (activeUpdateRef.current) {
            try {
                await activeUpdateRef.current.close();
            } catch {
                // Silencioso: el recurso ya puede haber sido liberado
            }
            activeUpdateRef.current = null;
        }
    }, []);

    const checkForUpdates = useCallback(
        async (options: { silent?: boolean } = {}) => {
            const { silent = false } = options;
            setStatus("checking");
            setError(null);

            try {
                await closeActiveUpdate();
                const update = await check();

                if (update) {
                    activeUpdateRef.current = update;
                    const info: UpdateInfo = {
                        version: update.version,
                        currentVersion: update.currentVersion,
                        date: update.date,
                        body: update.body,
                    };
                    setUpdateInfo(info);
                    setStatus("available");

                    if (silent) {
                        toast(
                            `Nueva versión v${update.version} de MyLAN disponible.`,
                            "default",
                        );
                    }
                } else {
                    activeUpdateRef.current = null;
                    setUpdateInfo(null);
                    setStatus(silent ? "idle" : "up-to-date");
                    if (!silent) {
                        toast(
                            "Tienes la versión más reciente de MyLAN.",
                            "default",
                        );
                    }
                }
            } catch (err) {
                const message =
                    err instanceof Error ? err.message : String(err);
                setError(message);
                setStatus("error");
                if (!silent) {
                    toast(
                        `Error al buscar actualizaciones: ${message}`,
                        "error",
                    );
                } else {
                    // En modo silencioso vuelve a idle para no bloquear la UI
                    setStatus("idle");
                }
            }
        },
        [closeActiveUpdate, toast],
    );

    const downloadAndInstall = useCallback(async () => {
        const update = activeUpdateRef.current;
        if (!update) {
            setError("No hay ninguna actualización activa para descargar.");
            setStatus("error");
            return;
        }

        setStatus("downloading");
        setError(null);
        setProgress({ downloaded: 0 });

        let totalLength: number | undefined;
        let downloadedBytes = 0;

        try {
            await update.downloadAndInstall((event) => {
                if (event.event === "Started") {
                    totalLength = event.data.contentLength;
                    setProgress({
                        total: totalLength,
                        downloaded: 0,
                        percent: totalLength ? 0 : undefined,
                    });
                } else if (event.event === "Progress") {
                    downloadedBytes += event.data.chunkLength;
                    const percent =
                        totalLength && totalLength > 0
                            ? Math.min(
                                  100,
                                  Math.round(
                                      (downloadedBytes / totalLength) * 100,
                                  ),
                              )
                            : undefined;
                    setProgress({
                        total: totalLength,
                        downloaded: downloadedBytes,
                        percent,
                    });
                } else if (event.event === "Finished") {
                    setProgress({
                        total: totalLength,
                        downloaded: downloadedBytes,
                        percent: 100,
                    });
                }
            });

            setStatus("ready");
            toast(
                "Actualización descargada e instalada. Reinicia para aplicar.",
                "success",
            );
        } catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            setError(message);
            setStatus("error");
            toast(`Error durante la actualización: ${message}`, "error");
        }
    }, [toast]);

    const restartApp = useCallback(async () => {
        try {
            await relaunch();
        } catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            toast(`No se pudo reiniciar automáticamente: ${message}`, "error");
        }
    }, [toast]);

    const dismissUpdate = useCallback(() => {
        void closeActiveUpdate();
        setStatus("idle");
        setUpdateInfo(null);
        setProgress(null);
    }, [closeActiveUpdate]);

    // Búsqueda silenciosa en el arranque (según preferencia en Settings)
    useEffect(() => {
        if (hasAutoCheckedRef.current) return;
        hasAutoCheckedRef.current = true;

        const timer = setTimeout(() => {
            getSettings()
                .then((s) => {
                    if (s.check_updates_on_startup !== false) {
                        void checkForUpdates({ silent: true });
                    }
                })
                .catch(() => {
                    // Si falla cargar settings (primer arranque / fallback), comprueba de forma segura
                    void checkForUpdates({ silent: true });
                });
        }, 3000);

        return () => clearTimeout(timer);
    }, [checkForUpdates]);

    return (
        <UpdaterContext.Provider
            value={{
                status,
                updateInfo,
                progress,
                error,
                checkForUpdates,
                downloadAndInstall,
                restartApp,
                dismissUpdate,
            }}
        >
            {children}
        </UpdaterContext.Provider>
    );
}

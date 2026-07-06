import { defineConfig, mergeConfig } from "vitest/config";
import viteConfig from "./vite.config";

// Extiende `vite.config.ts` para heredar plugins (react) + alias `@/`.
// CRITICAL: `vite.config.ts` exporta una config funcional async, por lo que
// `viteConfig(configEnv)` retorna `Promise<UserConfig>`. `mergeConfig` es
// sincrónico y no awaita Promises (itera `for...in` → cero props enumerables),
// así que el `await` explícito es obligatorio: sin él se pierden `plugins` y
// `resolve.alias`, y todos los tests de componentes fallan al compilar/resolver.
export default defineConfig(async (configEnv) => {
    const resolved = await viteConfig(configEnv); // await requerido
    return mergeConfig(
        resolved,
        defineConfig({
            test: {
                environment: "jsdom",
                setupFiles: ["./tests/setup.ts"],
                globals: true,
            },
        }),
    );
});
# MyLAN Desktop (Alpha)

App de escritorio **Tauri 2** (backend Rust + frontend React/TS/Vite/Tailwind/shadcn-ui) para
escanear tu red, ver dispositivos con puertos/servicios y exportar el inventario **sin
terminal**. Habla con `mylan-core/discovery/scanner/fingerprint/db` vía comandos Tauri IPC
directos (sin HTTP; `mylan-api` se difiere a Fase 7) y es dueña del ciclo de vida de la SQLite
local.

> El backend (`src-tauri/`) es un sub-paquete Cargo **independiente** del workspace Rust del
> repo (su propio `Cargo.lock`); no se compila con `cargo build --workspace` — usa los comandos
> de abajo.

## Requisitos de desarrollo

- **Node 20+** y npm (para el frontend y la CLI de Tauri).
- **Rust** con la toolchain pinneada del repo ([`rust-toolchain.toml`](../../rust-toolchain.toml),
  channel 1.96.0 + `rustfmt`/`clippy`); `rustup` la instala automáticamente.
- Un compilador de C (`cc`/`gcc`) para `rusqlite` (feature `bundled`).
- **Linux:** dependencias del sistema para WebKit/GTK (Tauri 2):

  ```bash
  sudo apt-get install -y \
    libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
    librsvg2-dev pkg-config build-essential
  ```

- **Windows:** MSVC Build Tools (los provee el runner de GitHub Actions; en local instala
  "Desktop development with C++" vía Visual Studio Build Tools). El discovery de red en Windows
  está degradado (sin ARP cache / ICMP no-root); `scan_target` TCP-connect sigue funcionando.

## Desarrollo

```bash
cd apps/desktop-tauri
npm ci            # instala dependencias del frontend
npm run dev       # sólo frontend (Vite, http://localhost:1420)
npm run tauri dev # levanta la app completa (frontend + ventana Tauri)
```

## Lint y build del frontend

```bash
cd apps/desktop-tauri
npm run lint   # eslint + tsc
npm run build  # tsc && vite build -> dist/
```

## Calidad del backend Tauri (fmt/clippy/build aparte del workspace)

```bash
cargo fmt --check  --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml
cargo clippy       --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml -- -D warnings
cargo build        --manifest-path apps/desktop-tauri/src-tauri/Cargo.toml
```

## Packaging (bundle instalable)

```bash
cd apps/desktop-tauri
npm run tauri build   # = tauri build (corre `npm run build` antes via beforeBuildCommand)
```

En Linux produce `src-tauri/target/release/bundle/{appimage,deb}/MyLAN_*.{AppImage,deb}`.
En Windows produce `src-tauri/target/release/bundle/{msi,nsis}/MyLAN_*.{msi,exe}`.
El bundle incluye las firmas OUI + reglas de `signatures/` (mapeadas a `resource_dir/signatures`
vía `bundle.resources` en `tauri.conf.json`), necesarias para `Fingerprint::load` empaquetado.

## Estructura

```
apps/desktop-tauri/
├── src/              frontend React (screens: Dashboard, Devices, DeviceDetail, Scans, Settings)
├── src-tauri/        backend Rust (#[tauri::command] wrappers sobre mylan-db/discovery/scanner)
│   ├── Cargo.toml    sub-paquete independiente (NO miembro del workspace)
│   └── tauri.conf.json
└── package.json
```

Véase el plan de Fase 4 en `.omc/plans/mylan-fase4-consensus.md` (Paso 7-8) para el detalle de
packaging, CI matrix y publish-safety.
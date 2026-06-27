# Contribuir a MyLAN

¡Gracias por tu interés! MyLAN es open-source y las contribuciones son bienvenidas.

## Antes de empezar

- Lee el [plan de implementación](MyLAN_plan_open_source.md) y el [ROADMAP](ROADMAP.md).
- Revisa la [guía de ética](docs/ethics.md): MyLAN es una herramienta **defensiva**.
- Busca un issue con la etiqueta `good first issue` si es tu primera contribución.

## Entorno de desarrollo

```bash
# Toolchain (pinneada en rust-toolchain.toml)
rustup show

# Build, formato, lints y tests
cargo build --workspace
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

La CI ejecuta exactamente esos cuatro comandos; déjalos en verde antes de abrir un PR.

## Estándares de código

- `cargo fmt` obligatorio; `clippy` sin warnings (`-D warnings`).
- Crates pequeños y enfocados; el dominio (`mylan-core`) sin I/O de plataforma.
- Patrones inmutables donde sea razonable; manejo de errores con `anyhow`/`thiserror`.
- Código específico de plataforma aislado tras `#[cfg(...)]` con impls portables por defecto.

## Commits y PRs

- Conventional Commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `perf:`, `ci:`.
- Un PR por cambio lógico; describe el "qué" y el "por qué".
- Añade tests para el comportamiento nuevo.

## Licencia de las contribuciones

Al contribuir aceptas que tu código se licencie bajo [AGPL-3.0-or-later](LICENSE).

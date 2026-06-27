//! `mylan-db` — persistencia SQLite local de MyLAN.
//!
//! Conexión `rusqlite` (feature `bundled`), migraciones SQL embebidas gobernadas por
//! `PRAGMA user_version`, y repositorios (upsert de dispositivos por identidad,
//! servicios, scans). Esquema según el plan §8.
//!
//! Estado: esqueleto (Paso 1). Implementación en Paso 3.

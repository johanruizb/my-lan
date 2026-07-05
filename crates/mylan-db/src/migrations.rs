//! Migraciones SQL embebidas gobernadas por `PRAGMA user_version`.
//!
//! Esquema según el plan §8 (`networks`, `devices`, `device_addresses`,
//! `services`, `scans`). Cada migración es idempotente (`CREATE ... IF NOT
//! EXISTS`) y se aplica exactamente una vez: se ejecutan las versiones cuyo
//! número es mayor que el `user_version` actual, que avanza en cada paso.

use rusqlite::Connection;

use crate::error::{map_sqlite, DbResult};

/// `(versión_objetivo, SQL)`. `user_version` parte de 0.
const MIGRATIONS: &[(u32, &str)] = &[
    (1, MIGRATION_V1),
    (2, MIGRATION_V2),
    (3, MIGRATION_V3),
    (4, MIGRATION_V4),
];

/// Esquema inicial completo del plan §8.
const MIGRATION_V1: &str = "\
CREATE TABLE IF NOT EXISTS networks (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  cidr TEXT NOT NULL,
  gateway_ip TEXT,
  dns_servers TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS devices (
  id TEXT PRIMARY KEY,
  network_id TEXT NOT NULL,
  primary_mac TEXT,
  primary_ip TEXT,
  hostname TEXT,
  display_name TEXT,
  vendor TEXT,
  manufacturer TEXT,
  model TEXT,
  device_type TEXT NOT NULL DEFAULT 'unknown',
  os_family TEXT,
  confidence INTEGER NOT NULL DEFAULT 0,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  is_trusted INTEGER NOT NULL DEFAULT 0,
  is_hidden INTEGER NOT NULL DEFAULT 0,
  notes TEXT,
  FOREIGN KEY(network_id) REFERENCES networks(id)
);

CREATE TABLE IF NOT EXISTS device_addresses (
  id TEXT PRIMARY KEY,
  device_id TEXT NOT NULL,
  ip TEXT,
  mac TEXT,
  interface_name TEXT,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  FOREIGN KEY(device_id) REFERENCES devices(id)
);

CREATE TABLE IF NOT EXISTS services (
  id TEXT PRIMARY KEY,
  device_id TEXT NOT NULL,
  protocol TEXT NOT NULL,
  port INTEGER NOT NULL,
  service_name TEXT,
  product TEXT,
  version TEXT,
  banner TEXT,
  state TEXT NOT NULL,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  FOREIGN KEY(device_id) REFERENCES devices(id)
);

CREATE TABLE IF NOT EXISTS scans (
  id TEXT PRIMARY KEY,
  network_id TEXT NOT NULL,
  scan_type TEXT NOT NULL,
  profile TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  summary_json TEXT,
  FOREIGN KEY(network_id) REFERENCES networks(id)
);

CREATE INDEX IF NOT EXISTS idx_devices_network_mac
  ON devices(network_id, primary_mac)
  WHERE primary_mac IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_devices_network_ip
  ON devices(network_id, primary_ip)
  WHERE primary_ip IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_device_addresses_device
  ON device_addresses(device_id);

CREATE INDEX IF NOT EXISTS idx_services_device
  ON services(device_id);

CREATE INDEX IF NOT EXISTS idx_scans_network
  ON scans(network_id);
";

/// v2 — Backstop de unicidad a nivel de esquema (defensa en profundidad, P5).
///
/// El upsert por identidad ya evita duplicados a nivel de aplicación; estos
/// índices `UNIQUE` convierten cualquier regresión futura de esa lógica en un
/// error duro en vez de un duplicado silencioso. Se mantiene `primary_ip` SIN
/// unicidad para tolerar solapamiento transitorio de IP por DHCP. El índice
/// no-único previo de MAC se reemplaza por su variante `UNIQUE`.
const MIGRATION_V2: &str = "\
DROP INDEX IF EXISTS idx_devices_network_mac;

CREATE UNIQUE INDEX IF NOT EXISTS uq_devices_network_mac
  ON devices(network_id, primary_mac)
  WHERE primary_mac IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS uq_services_device_proto_port
  ON services(device_id, protocol, port);
";

/// v3 — Origen del nombre de red (override de usuario vs auto-detección).
///
/// `name_source` distingue un nombre auto-derivado (SSID o CIDR-fallback,
/// `'auto'`) de una etiqueta editada por el usuario (`'user'`), para que una
/// re-detección de SSID no pise la etiqueta del usuario (precedencia de
/// override). Aditiva: `DEFAULT 'auto'` rellena las filas existentes.
const MIGRATION_V3: &str =
    "ALTER TABLE networks ADD COLUMN name_source TEXT NOT NULL DEFAULT 'auto';";

/// v4 — Línea de tiempo de eventos del agente + estado online de devices.
///
/// `events` persiste los diff events (`device_new`, `device_ip_changed`,
/// `device_offline`, `device_online`, `port_opened`) emitidos por el motor de
/// diff entre scans; la API lee de aquí y el canal WS es una vista en vivo de
/// la misma escritura (la DB es la fuente de verdad). `is_online` en `devices`
/// materializa el estado online/offline derivado del scan; el backfill one-shot
/// deriva el estado inicial del último scan por red (NO un blanket `DEFAULT 1`,
/// que produciría un estado todo-online falso y una tormenta de eventos
/// `device_offline` en el siguiente scan).
const MIGRATION_V4: &str = "\
CREATE TABLE IF NOT EXISTS events (
  id TEXT PRIMARY KEY,
  network_id TEXT NOT NULL,
  device_id TEXT,
  event_type TEXT NOT NULL,
  severity TEXT NOT NULL,
  message TEXT,
  data_json TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY(network_id) REFERENCES networks(id),
  FOREIGN KEY(device_id) REFERENCES devices(id)
);

CREATE INDEX IF NOT EXISTS idx_events_created_at
  ON events(created_at);

CREATE INDEX IF NOT EXISTS idx_events_network_device
  ON events(network_id, device_id);

ALTER TABLE devices ADD COLUMN is_online INTEGER NOT NULL DEFAULT 1;

UPDATE devices
SET is_online = (
  last_seen_at = (
    SELECT MAX(last_seen_at)
    FROM devices d2
    WHERE d2.network_id = devices.network_id
  )
);
";

/// Lee el `user_version` actual de la base de datos.
fn current_version(conn: &Connection) -> DbResult<u32> {
    let v: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(map_sqlite)?;
    Ok(u32::try_from(v).unwrap_or(0))
}

/// Aplica todas las migraciones pendientes (versiones > `user_version`).
///
/// Idempotente: llamarla de nuevo no hace nada cuando `user_version` ya alcanzó
/// la última. Cada migración se ejecuta con `execute_batch` y actualiza
/// `user_version` atómicamente.
pub fn run_migrations(conn: &Connection) -> DbResult<()> {
    let mut version = current_version(conn)?;
    for &(target, sql) in MIGRATIONS {
        if version >= target {
            continue;
        }
        conn.execute_batch(sql).map_err(map_sqlite)?;
        conn.execute_batch(&format!("PRAGMA user_version = {target};"))
            .map_err(map_sqlite)?;
        version = target;
    }
    Ok(())
}

/// Última versión de esquema soportada por este crate.
#[must_use]
pub fn latest_schema_version() -> u32 {
    MIGRATIONS.last().map_or(0, |(v, _)| *v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::connect;
    use rusqlite::Connection;

    fn connect_in_memory() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        conn
    }

    #[test]
    fn migration_is_idempotent() {
        let dir = tempfile::tempdir().expect("tmp");
        let path = dir.path().join("idem.db");
        {
            let conn = connect(&path).expect("open 1");
            run_migrations(&conn).expect("first migrate");
            assert_eq!(current_version(&conn).unwrap(), latest_schema_version());
        }
        {
            let conn = connect(&path).expect("open 2");
            // Segunda corrida: no debe errorar ni cambiar la versión.
            run_migrations(&conn).expect("second migrate");
            assert_eq!(current_version(&conn).unwrap(), latest_schema_version());
        }
    }

    #[test]
    fn schema_has_all_tables() {
        let conn = connect_in_memory();
        run_migrations(&conn).expect("migrate");
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        for expected in [
            "device_addresses",
            "devices",
            "events",
            "networks",
            "scans",
            "services",
        ] {
            assert!(
                tables.iter().any(|t| t == expected),
                "missing table {expected}"
            );
        }
    }

    #[test]
    fn unique_mac_backstop_rejects_raw_duplicate() {
        // Inserción cruda (saltándose el upsert) de dos devices con la misma
        // (network_id, primary_mac): el índice UNIQUE de v2 debe rechazar el 2º.
        let conn = connect_in_memory();
        run_migrations(&conn).expect("migrate");
        conn.execute(
            "INSERT INTO networks (id, name, cidr, created_at, updated_at)
             VALUES ('net-1','home','192.168.1.0/24','t0','t0')",
            [],
        )
        .unwrap();
        let insert = |id: &str| {
            conn.execute(
                "INSERT INTO devices (id, network_id, primary_mac, first_seen_at, last_seen_at)
                 VALUES (?1,'net-1','aa:bb:cc:dd:ee:ff','t0','t0')",
                [id],
            )
        };
        insert("dev-1").expect("first insert ok");
        assert!(
            insert("dev-2").is_err(),
            "UNIQUE(network_id,mac) debe rechazar duplicado"
        );
    }
}

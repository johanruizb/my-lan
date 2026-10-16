//! Repositorio de workspaces (`workspaces`).
//!
//! Gestión del ciclo de vida de los espacios de trabajo (Pilar 6 / Fase 10):
//! creación, lectura, actualización y eliminación segura con reasignación
//! automática al workspace `'default'`.

use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

use mylan_core::Workspace;

use crate::error::{map_sqlite, DbError, DbResult};

/// Estadísticas agregadas de un workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceStats {
    pub device_count: i64,
    pub network_count: i64,
    pub online_device_count: i64,
    pub open_findings_count: i64,
}

fn row_to_workspace(row: &Row<'_>) -> rusqlite::Result<Workspace> {
    Ok(Workspace {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        icon: row.get(3)?,
        color: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

/// Lista todos los workspaces registrados.
///
/// El workspace `'default'` siempre encabeza la lista para una experiencia
/// predecible en la interfaz, seguido del resto ordenado por nombre ascendente.
pub fn list_workspaces(conn: &Connection) -> DbResult<Vec<Workspace>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, icon, color, created_at, updated_at
             FROM workspaces
             ORDER BY CASE WHEN id = 'default' THEN 0 ELSE 1 END, name ASC",
        )
        .map_err(map_sqlite)?;

    let rows = stmt
        .query_map([], row_to_workspace)
        .map_err(map_sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_sqlite)?;

    Ok(rows)
}

/// Obtiene un workspace por su `id`. Devuelve `None` si no existe.
pub fn get_workspace(conn: &Connection, id: &str) -> DbResult<Option<Workspace>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, icon, color, created_at, updated_at
             FROM workspaces
             WHERE id = ?1",
        )
        .map_err(map_sqlite)?;

    let mut rows = stmt.query_map([id], row_to_workspace).map_err(map_sqlite)?;

    match rows.next() {
        Some(row) => Ok(Some(row.map_err(map_sqlite)?)),
        None => Ok(None),
    }
}

/// Inserta un nuevo workspace.
pub fn create_workspace(conn: &Connection, ws: &Workspace) -> DbResult<()> {
    conn.execute(
        "INSERT INTO workspaces (id, name, description, icon, color, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            ws.id,
            ws.name,
            ws.description,
            ws.icon,
            ws.color,
            ws.created_at,
            ws.updated_at,
        ],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

/// Actualiza los metadatos de un workspace (nombre, descripción, icono, color y updated_at).
pub fn update_workspace(conn: &Connection, ws: &Workspace) -> DbResult<()> {
    conn.execute(
        "UPDATE workspaces
         SET name = ?2, description = ?3, icon = ?4, color = ?5, updated_at = ?6
         WHERE id = ?1",
        params![
            ws.id,
            ws.name,
            ws.description,
            ws.icon,
            ws.color,
            ws.updated_at,
        ],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

/// Elimina un workspace por su `id`.
///
/// Protege el workspace `'default'` (devolverá [`DbError::Domain`] si se intenta borrar).
/// Para evitar violaciones de clave foránea o pérdida de histórico, reasigna todos
/// los registros vinculados (redes, scans, devices, eventos, hallazgos y mediciones)
/// al workspace `'default'` antes de eliminar la fila.
pub fn delete_workspace(conn: &Connection, id: &str) -> DbResult<()> {
    if id == "default" {
        return Err(DbError::Domain(
            "El workspace por defecto 'default' no puede ser eliminado.".to_string(),
        ));
    }

    // Reasignación preventiva al workspace por defecto
    conn.execute(
        "UPDATE networks SET workspace_id = 'default' WHERE workspace_id = ?1",
        [id],
    )
    .map_err(map_sqlite)?;
    conn.execute(
        "UPDATE scans SET workspace_id = 'default' WHERE workspace_id = ?1",
        [id],
    )
    .map_err(map_sqlite)?;
    conn.execute(
        "UPDATE devices SET workspace_id = 'default' WHERE workspace_id = ?1",
        [id],
    )
    .map_err(map_sqlite)?;
    conn.execute(
        "UPDATE events SET workspace_id = 'default' WHERE workspace_id = ?1",
        [id],
    )
    .map_err(map_sqlite)?;
    conn.execute(
        "UPDATE security_findings SET workspace_id = 'default' WHERE workspace_id = ?1",
        [id],
    )
    .map_err(map_sqlite)?;
    conn.execute(
        "UPDATE network_health_samples SET workspace_id = 'default' WHERE workspace_id = ?1",
        [id],
    )
    .map_err(map_sqlite)?;
    conn.execute(
        "UPDATE speed_tests SET workspace_id = 'default' WHERE workspace_id = ?1",
        [id],
    )
    .map_err(map_sqlite)?;

    // Borrado definitivo
    conn.execute("DELETE FROM workspaces WHERE id = ?1", [id])
        .map_err(map_sqlite)?;
    Ok(())
}

/// Asocia explícitamente una red existente a un workspace dado.
pub fn assign_network_to_workspace(
    conn: &Connection,
    network_id: &str,
    workspace_id: &str,
) -> DbResult<()> {
    conn.execute(
        "UPDATE networks SET workspace_id = ?2 WHERE id = ?1",
        params![network_id, workspace_id],
    )
    .map_err(map_sqlite)?;
    Ok(())
}

/// Obtiene el `workspace_id` de una red dada (`None` si no existe la red).
pub fn get_network_workspace(conn: &Connection, network_id: &str) -> DbResult<Option<String>> {
    let mut stmt = conn
        .prepare("SELECT workspace_id FROM networks WHERE id = ?1")
        .map_err(map_sqlite)?;
    let mut rows = stmt
        .query_map([network_id], |r| r.get::<_, String>(0))
        .map_err(map_sqlite)?;
    match rows.next() {
        Some(r) => Ok(Some(r.map_err(map_sqlite)?)),
        None => Ok(None),
    }
}

/// Calcula estadísticas rápidas para la tarjeta o encabezado del workspace.
pub fn get_workspace_stats(conn: &Connection, workspace_id: &str) -> DbResult<WorkspaceStats> {
    let network_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM networks WHERE workspace_id = ?1",
            [workspace_id],
            |r| r.get(0),
        )
        .map_err(map_sqlite)?;

    let device_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM devices WHERE workspace_id = ?1",
            [workspace_id],
            |r| r.get(0),
        )
        .map_err(map_sqlite)?;

    let online_device_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM devices WHERE workspace_id = ?1 AND is_online = 1",
            [workspace_id],
            |r| r.get(0),
        )
        .map_err(map_sqlite)?;

    let open_findings_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM security_findings WHERE workspace_id = ?1 AND status = 'open'",
            [workspace_id],
            |r| r.get(0),
        )
        .map_err(map_sqlite)?;

    Ok(WorkspaceStats {
        device_count,
        network_count,
        online_device_count,
        open_findings_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::run_migrations;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn test_list_workspaces_has_default_first() {
        let conn = setup_db();
        let ws = Workspace::new("ws-b", "Beta Office", "2026-10-16T10:00:00Z");
        create_workspace(&conn, &ws).unwrap();
        let ws2 = Workspace::new("ws-a", "Alpha Office", "2026-10-16T10:00:00Z");
        create_workspace(&conn, &ws2).unwrap();

        let list = list_workspaces(&conn).unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].id, "default");
        assert_eq!(list[1].name, "Alpha Office");
        assert_eq!(list[2].name, "Beta Office");
    }

    #[test]
    fn test_crud_workspace() {
        let conn = setup_db();
        let mut ws = Workspace::new("ws-test", "Oficina Test", "2026-10-16T10:00:00Z");
        ws.description = Some("Descripción de prueba".into());
        ws.icon = "building".into();
        ws.color = "#8b5cf6".into();

        create_workspace(&conn, &ws).unwrap();

        let fetched = get_workspace(&conn, "ws-test").unwrap().expect("found");
        assert_eq!(fetched.name, "Oficina Test");
        assert_eq!(fetched.icon, "building");
        assert_eq!(fetched.color, "#8b5cf6");

        // Actualizar
        ws.name = "Oficina Renombrada".into();
        ws.color = "#ef4444".into();
        update_workspace(&conn, &ws).unwrap();

        let updated = get_workspace(&conn, "ws-test").unwrap().expect("found");
        assert_eq!(updated.name, "Oficina Renombrada");
        assert_eq!(updated.color, "#ef4444");
    }

    #[test]
    fn test_cannot_delete_default_workspace() {
        let conn = setup_db();
        let res = delete_workspace(&conn, "default");
        assert!(res.is_err());
    }

    #[test]
    fn test_delete_workspace_reassigns_elements_to_default() {
        let conn = setup_db();
        let ws = Workspace::new("ws-corp", "Corporativo", "2026-10-16T10:00:00Z");
        create_workspace(&conn, &ws).unwrap();

        // Insertar red asociada al workspace corporativo
        conn.execute(
            "INSERT INTO networks (id, name, cidr, workspace_id, created_at, updated_at)
             VALUES ('net-corp', 'Corp LAN', '10.50.0.0/16', 'ws-corp', 't0', 't0')",
            [],
        )
        .unwrap();

        // Borrar el workspace corporativo
        delete_workspace(&conn, "ws-corp").unwrap();

        // El workspace ya no existe
        assert!(get_workspace(&conn, "ws-corp").unwrap().is_none());

        // La red fue reasignada al default
        let ws_of_net = get_network_workspace(&conn, "net-corp").unwrap().unwrap();
        assert_eq!(ws_of_net, "default");
    }

    #[test]
    fn test_workspace_stats() {
        let conn = setup_db();
        let ws = Workspace::new("ws-branch", "Sucursal", "2026-10-16T10:00:00Z");
        create_workspace(&conn, &ws).unwrap();

        conn.execute(
            "INSERT INTO networks (id, name, cidr, workspace_id, created_at, updated_at)
             VALUES ('net-b', 'Branch LAN', '172.16.0.0/24', 'ws-branch', 't0', 't0')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO devices (id, network_id, primary_ip, is_online, workspace_id, first_seen_at, last_seen_at)
             VALUES ('dev-1', 'net-b', '172.16.0.10', 1, 'ws-branch', 't0', 't0')",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO devices (id, network_id, primary_ip, is_online, workspace_id, first_seen_at, last_seen_at)
             VALUES ('dev-2', 'net-b', '172.16.0.11', 0, 'ws-branch', 't0', 't0')",
            [],
        )
        .unwrap();

        let stats = get_workspace_stats(&conn, "ws-branch").unwrap();
        assert_eq!(stats.network_count, 1);
        assert_eq!(stats.device_count, 2);
        assert_eq!(stats.online_device_count, 1);
        assert_eq!(stats.open_findings_count, 0);
    }
}

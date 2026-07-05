//! Test del lifecycle del token (AC-7: generate → persist `0600` → reload → rotate).

use mylan_api::{generate_token, load_or_create_token, rotate_token};

#[test]
fn generate_persist_reload_rotate() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("api-token");
    // generate: token no vacío.
    let t0 = generate_token().unwrap();
    assert!(!t0.is_empty());
    // load_or_create: el fichero no existe → crea.
    let t1 = load_or_create_token(&path).unwrap();
    assert!(path.exists());
    assert_ne!(t0, t1, "load_or_create genera su propio token");
    // reload: el fichero existe → devuelve el mismo.
    let t2 = load_or_create_token(&path).unwrap();
    assert_eq!(t1, t2, "reload devuelve el mismo token");
    // rotate: genera uno nuevo y sobrescribe.
    let t3 = rotate_token(&path).unwrap();
    assert_ne!(t1, t3, "rotación cambia el token");
    let t4 = load_or_create_token(&path).unwrap();
    assert_eq!(t3, t4, "reload tras rotate devuelve el nuevo");
}

#[cfg(unix)]
#[test]
fn persisted_token_has_0600_perms() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("api-token");
    load_or_create_token(&path).unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "0600 tras crear");
    rotate_token(&path).unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "0600 tras rotar");
}

#[cfg(unix)]
#[test]
fn rotate_reasserts_0600_after_external_chmod() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("api-token");
    load_or_create_token(&path).unwrap();
    // Corromper permisos a 0644 (simula un admin que los abrió).
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    // rotate debe reafirmar 0600 (write_token_file setea permisos siempre).
    rotate_token(&path).unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "rotate reafirma 0600");
}

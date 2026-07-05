//! AC-2: parse de `mylan-agent.toml` — `interval_secs`, `networks`, `api_port`.

use mylan_agent::AgentConfig;

#[test]
fn parse_sample_config() {
    let toml = r#"
interval_secs = 60
api_port = 43117

[[networks]]
cidr = "192.168.1.0/24"
profile = "quick"

[[networks]]
cidr = "10.0.0.0/24"
profile = "normal"
"#;
    let cfg = AgentConfig::parse(toml).expect("parse");
    assert_eq!(cfg.interval_secs, 60);
    assert_eq!(cfg.api_port, 43117);
    assert_eq!(cfg.networks.len(), 2);
    assert_eq!(cfg.networks[0].cidr, "192.168.1.0/24");
    assert_eq!(cfg.networks[0].profile, mylan_core::ScanProfile::Quick);
    assert_eq!(cfg.networks[1].cidr, "10.0.0.0/24");
    assert_eq!(cfg.networks[1].profile, mylan_core::ScanProfile::Normal);
}

#[test]
fn parse_rejects_missing_interval_secs() {
    let toml = r#"
api_port = 43117
networks = []
"#;
    assert!(AgentConfig::parse(toml).is_err(), "falta interval_secs");
}

#[test]
fn db_path_unset_falls_back_to_default() {
    let toml = r#"
interval_secs = 30
api_port = 43117
networks = []
"#;
    let cfg = AgentConfig::parse(toml).expect("parse");
    assert!(cfg.db_path.is_none(), "db_path no seteado en el TOML");
    // db_path() resuelve via default_db_path(); puede ser None sin HOME, pero
    // no debe paniquear.
    let _ = cfg.db_path();
}

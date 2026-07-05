//! AC-1: lifecycle del agent — `CancellationToken` lo hace salir en <2s con
//! exit 0 (single-process ADR-4: agent + API en un proceso).

use std::time::Duration;

use tokio_util::sync::CancellationToken;

use mylan_agent::{AgentConfig, run_agent_with_cancel};

#[tokio::test]
async fn cancel_exits_promptly_with_ok() {
    let config = AgentConfig {
        interval_secs: 3600,
        networks: vec![],
        api_port: 0,
        db_path: None,
    };
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("mylan.db");
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;
        cancel_clone.cancel();
    });
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        run_agent_with_cancel(&config, &db_path, 0, "test-token", cancel),
    )
    .await
    .expect("run_agent_with_cancel debe terminar <2s tras cancel");
    assert!(result.is_ok(), "scheduler debe retornar Ok(()) al cancelarse");
}
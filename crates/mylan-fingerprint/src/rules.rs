//! Motor de reglas YAML para fingerprinting de `device_type` + `confidence`.
//!
//! Formato (plan §7.4):
//! ```yaml
//! id: camera_rtsp_generic
//! match:
//!   any:                # OR sobre los matchers (alternativa: `all` = AND)
//!     - port: 554
//!       service: rtsp
//!     - mdns_contains: "_rtsp"
//!     - upnp_device_type_contains: "MediaServer"
//! score:
//!   device_type: camera
//!   confidence: 75
//! ```
//!
//! Cada [`Matcher`] expone las señales soportadas y casa contra `&[Observation]`:
//! - `port`: algún `tcp.ports` == puerto.
//! - `service`: algún hint cuyo valor contiene la cadena (mDNS/SSDP/banner).
//! - `mdns_contains`: algún hint `mdns.*` cuyo valor contiene la cadena.
//! - `upnp_device_type_contains`: algún hint `ssdp.*` cuyo valor contiene la cadena.
//! - `is_gateway`: alguna observación marca `network.role == "gateway"`.
//!
//! Un matcher casa si **todas** sus campos presentes casan (AND interno). La
//! rama `any` casa si cualquier matcher casa; `all` exige todos. La precedencia
//! de confidence la decide `Device::apply_classification` en el enricher.

use std::path::Path;

use mylan_core::{Confidence, DeviceType, Observation};
use serde::Deserialize;

use crate::error::FingerprintError;

/// Conjunto de reglas cargadas desde `signatures/device-rules/*.yaml`.
#[derive(Debug, Clone, Default)]
pub struct RuleSet {
    rules: Vec<Rule>,
}

/// Una regla de fingerprinting.
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub matcher: Match,
    pub device_type: DeviceType,
    pub confidence: Confidence,
}

/// Estructura de match: `any` (OR) y/o `all` (AND) sobre [`Matcher`].
#[derive(Debug, Clone, Default)]
pub struct Match {
    pub any: Vec<Matcher>,
    pub all: Vec<Matcher>,
}

/// Una condición concreta contra las observaciones.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Matcher {
    pub port: Option<u16>,
    pub service: Option<String>,
    pub mdns_contains: Option<String>,
    pub upnp_device_type_contains: Option<String>,
    pub is_gateway: Option<bool>,
}

impl Match {
    /// `true` si no tiene matchers definidos.
    pub fn is_empty(&self) -> bool {
        self.any.is_empty() && self.all.is_empty()
    }

    /// Evalúa el match contra las observaciones.
    pub fn matches(&self, observations: &[Observation]) -> bool {
        let any_ok = self.any.is_empty() || self.any.iter().any(|m| m.matches(observations));
        let all_ok = self.all.is_empty() || self.all.iter().all(|m| m.matches(observations));
        // Si ambos vacíos -> sin match. Si solo `any` -> any_ok. Si solo `all` -> all_ok.
        // Si ambos presentes -> any_ok && all_ok.
        if self.any.is_empty() && self.all.is_empty() {
            false
        } else if self.all.is_empty() {
            any_ok
        } else if self.any.is_empty() {
            all_ok
        } else {
            any_ok && all_ok
        }
    }
}

impl Matcher {
    /// `true` si todos los campos presentes casan con alguna observación.
    pub fn matches(&self, observations: &[Observation]) -> bool {
        let mut ok = true;
        if let Some(port) = self.port {
            ok &= observations.iter().any(|o| {
                o.hints
                    .get("tcp.ports")
                    .is_some_and(|v| v == &port.to_string())
            });
        }
        if let Some(service) = &self.service {
            let needle = service.to_lowercase();
            ok &= observations
                .iter()
                .flat_map(|o| o.hints.values())
                .any(|v| v.to_lowercase().contains(&needle));
        }
        if let Some(needle) = &self.mdns_contains {
            let n = needle.to_lowercase();
            ok &= observations.iter().any(|o| {
                o.hints
                    .iter()
                    .filter(|(k, _)| k.starts_with("mdns."))
                    .any(|(_, v)| v.to_lowercase().contains(&n))
            });
        }
        if let Some(needle) = &self.upnp_device_type_contains {
            let n = needle.to_lowercase();
            ok &= observations.iter().any(|o| {
                o.hints
                    .iter()
                    .filter(|(k, _)| k.starts_with("ssdp."))
                    .any(|(_, v)| v.to_lowercase().contains(&n))
            });
        }
        if let Some(true) = self.is_gateway {
            ok &= observations
                .iter()
                .any(|o| o.hints.get("network.role").is_some_and(|v| v == "gateway"));
        }
        ok
    }
}

impl Rule {
    /// `true` si la regla casa con las observaciones.
    pub fn matches(&self, observations: &[Observation]) -> bool {
        self.matcher.matches(observations)
    }
}

impl RuleSet {
    /// Crea un conjunto vacío.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Carga todas las reglas `*.yaml` de un directorio (no recursivo).
    pub fn load_dir(dir: &Path) -> Result<Self, FingerprintError> {
        let mut rules = Vec::new();
        if !dir.exists() {
            return Ok(Self { rules });
        }
        let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                let raw = std::fs::read_to_string(&path)?;
                let rule: RuleYaml =
                    serde_yaml_ng::from_str(&raw).map_err(|e| FingerprintError::RuleLoad {
                        path: path.display().to_string(),
                        message: e.to_string(),
                    })?;
                rules.push(rule.into_rule());
            }
        }
        Ok(Self { rules })
    }

    /// Número de reglas cargadas.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// `true` si no hay reglas.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Reglas cargadas (orden de carga, determinista).
    #[must_use]
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Evalúa las reglas contra las observaciones y devuelve la mejor
    /// clasificación (mayor confidence; empate => primera en orden de carga).
    #[must_use]
    pub fn evaluate(&self, observations: &[Observation]) -> Option<(DeviceType, Confidence)> {
        self.rules
            .iter()
            .filter(|r| r.matches(observations))
            .map(|r| (r.device_type, r.confidence))
            .max_by_key(|(_, c)| c.score())
    }
}

// --- Deserialización YAML -------------------------------------------------

#[derive(Debug, Deserialize)]
struct RuleYaml {
    id: String,
    #[serde(default)]
    r#match: MatchYaml,
    score: ScoreYaml,
}

impl RuleYaml {
    fn into_rule(self) -> Rule {
        Rule {
            id: self.id,
            matcher: Match {
                any: self.r#match.any.into_iter().map(Into::into).collect(),
                all: self.r#match.all.into_iter().map(Into::into).collect(),
            },
            device_type: self.score.device_type.into(),
            confidence: Confidence::new(self.score.confidence),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct MatchYaml {
    #[serde(default)]
    any: Vec<MatcherYaml>,
    #[serde(default)]
    all: Vec<MatcherYaml>,
}

#[derive(Debug, Default, Deserialize)]
struct MatcherYaml {
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    service: Option<String>,
    #[serde(default)]
    mdns_contains: Option<String>,
    #[serde(default)]
    upnp_device_type_contains: Option<String>,
    #[serde(default)]
    is_gateway: Option<bool>,
}

impl From<MatcherYaml> for Matcher {
    fn from(y: MatcherYaml) -> Self {
        Self {
            port: y.port,
            service: y.service,
            mdns_contains: y.mdns_contains,
            upnp_device_type_contains: y.upnp_device_type_contains,
            is_gateway: y.is_gateway,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ScoreYaml {
    device_type: DeviceTypeSerde,
    confidence: u8,
}

/// Wrapper para deserializar `DeviceType` desde snake_case (ya serializa así).
/// serde_yaml_ng necesita que el enum derive Deserialize; reutilizamos el de
/// mylan-core vía su serialización, pero como YAML usa las mismas variantes
/// snake_case, parseamos a través de un enum espejo para evitar acoplar la
/// representación interna de mylan-core.
#[derive(Debug, Clone, Copy, Deserialize)]
enum DeviceTypeSerde {
    #[serde(rename = "router")]
    Router,
    #[serde(rename = "phone")]
    Phone,
    #[serde(rename = "laptop")]
    Laptop,
    #[serde(rename = "desktop")]
    Desktop,
    #[serde(rename = "tv")]
    Tv,
    #[serde(rename = "printer")]
    Printer,
    #[serde(rename = "camera")]
    Camera,
    #[serde(rename = "nas")]
    Nas,
    #[serde(rename = "console")]
    Console,
    #[serde(rename = "iot")]
    Iot,
    #[serde(rename = "tablet")]
    Tablet,
    #[serde(rename = "unknown")]
    Unknown,
}

impl From<DeviceTypeSerde> for DeviceType {
    fn from(s: DeviceTypeSerde) -> Self {
        match s {
            DeviceTypeSerde::Router => DeviceType::Router,
            DeviceTypeSerde::Phone => DeviceType::Phone,
            DeviceTypeSerde::Laptop => DeviceType::Laptop,
            DeviceTypeSerde::Desktop => DeviceType::Desktop,
            DeviceTypeSerde::Tv => DeviceType::Tv,
            DeviceTypeSerde::Printer => DeviceType::Printer,
            DeviceTypeSerde::Camera => DeviceType::Camera,
            DeviceTypeSerde::Nas => DeviceType::Nas,
            DeviceTypeSerde::Console => DeviceType::Console,
            DeviceTypeSerde::Iot => DeviceType::Iot,
            DeviceTypeSerde::Tablet => DeviceType::Tablet,
            DeviceTypeSerde::Unknown => DeviceType::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mylan_core::Source;
    use std::net::IpAddr;

    fn ip(s: &str) -> IpAddr {
        s.parse().expect("valid ip")
    }

    fn obs_mdns(service: &str) -> Observation {
        Observation::new(Source::Mdns)
            .with_ip(ip("192.168.1.10"))
            .with_hint("mdns.service", service)
    }

    fn obs_ssdp(st: &str) -> Observation {
        Observation::new(Source::Ssdp)
            .with_ip(ip("192.168.1.11"))
            .with_hint("ssdp.st", st)
    }

    fn obs_tcp(port: u16) -> Observation {
        Observation::new(Source::TcpPing)
            .with_ip(ip("192.168.1.12"))
            .with_hint("tcp.ports", port.to_string())
    }

    fn camera_rule() -> Rule {
        Rule {
            id: "camera_rtsp_generic".to_string(),
            matcher: Match {
                any: vec![
                    Matcher {
                        port: Some(554),
                        service: Some("rtsp".to_string()),
                        ..Default::default()
                    },
                    Matcher {
                        mdns_contains: Some("_rtsp".to_string()),
                        ..Default::default()
                    },
                    Matcher {
                        upnp_device_type_contains: Some("MediaServer".to_string()),
                        ..Default::default()
                    },
                ],
                all: vec![],
            },
            device_type: DeviceType::Camera,
            confidence: Confidence::new(75),
        }
    }

    #[test]
    fn camera_rule_fires_on_mdns_rtsp() {
        let rules = RuleSet {
            rules: vec![camera_rule()],
        };
        let obs = [obs_mdns("_rtsp._tcp")];
        let (dt, conf) = rules.evaluate(&obs).expect("match");
        assert_eq!(dt, DeviceType::Camera);
        assert_eq!(conf.score(), 75);
    }

    #[test]
    fn camera_rule_fires_on_ssdp_mediaserver() {
        let rules = RuleSet {
            rules: vec![camera_rule()],
        };
        let obs = [obs_ssdp("urn:schemas-upnp-org:device:MediaServer:1")];
        assert_eq!(
            rules.evaluate(&obs).map(|(d, _)| d),
            Some(DeviceType::Camera)
        );
    }

    #[test]
    fn camera_rule_does_not_fire_on_unrelated_mdns() {
        let rules = RuleSet {
            rules: vec![camera_rule()],
        };
        let obs = [obs_mdns("_ipp._tcp")];
        assert!(rules.evaluate(&obs).is_none());
    }

    #[test]
    fn highest_confidence_wins_across_rules() {
        let camera = Rule {
            id: "camera".to_string(),
            matcher: Match {
                any: vec![Matcher {
                    mdns_contains: Some("_rtsp".to_string()),
                    ..Default::default()
                }],
                all: vec![],
            },
            device_type: DeviceType::Camera,
            confidence: Confidence::new(75),
        };
        let iot = Rule {
            id: "iot".to_string(),
            matcher: Match {
                any: vec![Matcher {
                    mdns_contains: Some("_rtsp".to_string()),
                    ..Default::default()
                }],
                all: vec![],
            },
            device_type: DeviceType::Iot,
            confidence: Confidence::new(40),
        };
        let rules = RuleSet {
            rules: vec![iot, camera],
        };
        let obs = [obs_mdns("_rtsp._tcp")];
        let (dt, conf) = rules.evaluate(&obs).expect("match");
        assert_eq!(dt, DeviceType::Camera);
        assert_eq!(conf.score(), 75);
    }

    #[test]
    fn all_branch_requires_every_matcher() {
        let rule = Rule {
            id: "printer_ipp".to_string(),
            matcher: Match {
                any: vec![],
                all: vec![
                    Matcher {
                        port: Some(631),
                        ..Default::default()
                    },
                    Matcher {
                        mdns_contains: Some("_ipp".to_string()),
                        ..Default::default()
                    },
                ],
            },
            device_type: DeviceType::Printer,
            confidence: Confidence::new(80),
        };
        let rules = RuleSet { rules: vec![rule] };
        // Only port -> no match (all requires both).
        assert!(rules.evaluate(&[obs_tcp(631)]).is_none());
        // Port + mDNS _ipp -> match.
        let obs = [obs_tcp(631), obs_mdns("_ipp._tcp")];
        assert_eq!(
            rules.evaluate(&obs).map(|(d, _)| d),
            Some(DeviceType::Printer)
        );
    }

    #[test]
    fn is_gateway_matcher_fires_on_hint() {
        let rule = Rule {
            id: "router_gateway".to_string(),
            matcher: Match {
                any: vec![Matcher {
                    is_gateway: Some(true),
                    ..Default::default()
                }],
                all: vec![],
            },
            device_type: DeviceType::Router,
            confidence: Confidence::new(70),
        };
        let rules = RuleSet { rules: vec![rule] };
        let obs = [Observation::new(Source::ArpCache)
            .with_ip(ip("192.168.1.1"))
            .with_hint("network.role", "gateway")];
        assert_eq!(
            rules.evaluate(&obs).map(|(d, _)| d),
            Some(DeviceType::Router)
        );
    }

    #[test]
    fn loads_camera_rule_yaml_from_signatures() {
        let dir = std::path::Path::new("../../signatures/device-rules");
        let rules = RuleSet::load_dir(dir).expect("load rules");
        assert!(!rules.is_empty(), "at least camera_rtsp_generic must load");
        let camera = rules
            .rules()
            .iter()
            .find(|r| r.id == "camera_rtsp_generic")
            .expect("camera_rtsp_generic present");
        let obs = [obs_mdns("_rtsp._tcp")];
        assert!(camera.matches(&obs));
        assert_eq!(camera.device_type, DeviceType::Camera);
        assert_eq!(camera.confidence.score(), 75);
    }
}

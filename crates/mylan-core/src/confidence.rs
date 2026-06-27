//! Puntuación de confianza (`Confidence`) de una clasificación de dispositivo.

use serde::{Deserialize, Serialize};

/// Confianza en una inferencia (vendor / `device_type`), acotada a `0..=100`.
///
/// Serializa como el entero subyacente (la DB la guarda como `INTEGER`). El orden
/// natural permite la precedencia de merge: gana la mayor confianza.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Confidence(u8);

impl Confidence {
    /// Confianza mínima (sin evidencia).
    pub const NONE: Self = Self(0);
    /// Confianza máxima.
    pub const MAX: Self = Self(100);

    /// Construye una confianza acotando el valor a `0..=100`.
    #[must_use]
    pub const fn new(score: u8) -> Self {
        Self(if score > 100 { 100 } else { score })
    }

    /// Valor numérico `0..=100`.
    #[must_use]
    pub const fn score(self) -> u8 {
        self.0
    }
}

impl Default for Confidence {
    fn default() -> Self {
        Self::NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_above_max() {
        assert_eq!(Confidence::new(250).score(), 100);
        assert_eq!(Confidence::new(75).score(), 75);
    }

    #[test]
    fn defaults_to_none() {
        assert_eq!(Confidence::default(), Confidence::NONE);
        assert_eq!(Confidence::NONE.score(), 0);
    }

    #[test]
    fn orders_by_score() {
        assert!(Confidence::new(75) > Confidence::new(40));
        assert!(Confidence::MAX > Confidence::new(99));
    }

    #[test]
    fn serde_round_trip_as_integer() {
        let json = serde_json::to_string(&Confidence::new(86)).expect("serialize");
        assert_eq!(json, "86");
        let back: Confidence = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, Confidence::new(86));
    }
}

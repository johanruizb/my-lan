//! Motor criptografico Zero-Knowledge para MyLAN Sync.
//!
//! Implementa:
//! - Derivacion de clave segura mediante Argon2id (RFC 9106)
//! - Generacion y restauracion de mnemotecnicos BIP-39 (12 palabras)
//! - Cifrado autenticado AEAD con XChaCha20-Poly1305 (nonce extendido de 192 bits)
//! - Formato de paquete binario con cabecera fija de 48 bytes e integridad autenticada

use argon2::{Algorithm, Argon2, Params, Version};
use bip39::{Language, Mnemonic};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use sha2::{Digest, Sha256};
use thiserror::Error;
use zeroize::Zeroizing;

pub const MAGIC_BYTES: [u8; 4] = *b"MYLN";
pub const CURRENT_VERSION: u8 = 0x01;
pub const KDF_ARGON2ID: u8 = 0x01;
pub const CIPHER_XCHACHA20_POLY1305: u8 = 0x01;
pub const RESERVED_BYTE: u8 = 0x00;
pub const HEADER_LEN: usize = 48;
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 24;
pub const KEY_LEN: usize = 32;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Error de derivacion de clave KDF: {0}")]
    KdfError(String),

    #[error("Error de cifrado simetrico: {0}")]
    EncryptionError(String),

    #[error("Error de descifrado / integridad (contrasena incorrecta o datos alterados)")]
    DecryptionFailed,

    #[error("Cabecera de paquete invalida: longitud insuficiente ({0} bytes)")]
    HeaderTooShort(usize),

    #[error("Magic bytes invalidos en cabecera: {0:?}")]
    InvalidMagic([u8; 4]),

    #[error("Version de protocolo no soportada: {0}")]
    UnsupportedVersion(u8),

    #[error("Algoritmo KDF no soportado: {0}")]
    UnsupportedKdf(u8),

    #[error("Algoritmo de cifrado no soportado: {0}")]
    UnsupportedCipher(u8),

    #[error("Mnemotecnico BIP-39 invalido: {0}")]
    MnemonicError(String),

    #[error("Fallo de generador de numeros aleatorios: {0}")]
    RandomError(String),
}

/// Genera bytes criptograficos aleatorios utilizando el CSPRNG del sistema operativo.
pub fn random_bytes<const N: usize>() -> Result<[u8; N], CryptoError> {
    let mut buf = [0u8; N];
    getrandom::getrandom(&mut buf)
        .map_err(|e| CryptoError::RandomError(format!("getrandom fallo: {e}")))?;
    Ok(buf)
}

/// Deriva una clave simetrica de 32 bytes (256 bits) usando Argon2id con salt aleatorio.
///
/// Parametros de seguridad:
/// - Memoria: 64 MiB (65536 KiB)
/// - Iteraciones de tiempo: 3
/// - Paralelismo: 1 canal
pub fn derive_key_argon2id(
    passphrase: &str,
    salt: &[u8; SALT_LEN],
) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
    let params = Params::new(65536, 3, 1, Some(KEY_LEN))
        .map_err(|e| CryptoError::KdfError(format!("Parametros invalidos de Argon2: {e}")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, key.as_mut())
        .map_err(|e| CryptoError::KdfError(format!("Fallo hash_password_into: {e}")))?;

    Ok(key)
}

/// Genera una frase mnemotecnica de 12 palabras (BIP-39) y deriva su clave correspondiente.
pub fn generate_mnemonic_and_key() -> Result<(String, Zeroizing<[u8; KEY_LEN]>), CryptoError> {
    let entropy = random_bytes::<16>()?;
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .map_err(|e| CryptoError::MnemonicError(e.to_string()))?;

    let phrase = mnemonic.to_string();
    let key = mnemonic_to_key(&phrase)?;
    Ok((phrase, key))
}

/// Deriva la clave simetrica a partir de una frase mnemotecnica BIP-39 de 12 palabras.
pub fn mnemonic_to_key(phrase: &str) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
    let mnemonic = Mnemonic::parse_in(Language::English, phrase.trim())
        .map_err(|e| CryptoError::MnemonicError(e.to_string()))?;

    // Seed de 64 bytes segun especificacion BIP-39
    let seed = mnemonic.to_seed("");
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    // Derivamos los 32 bytes de clave simetrica mediante SHA-256 de la seed BIP-39
    let hash = Sha256::digest(seed);
    key.copy_from_slice(&hash);
    Ok(key)
}

/// Calcula un hash de verificacion SHA-256 (hex) de la clave para comprobar contrasenas sin almacenar la clave en claro.
pub fn compute_key_verification_hash(key: &[u8; KEY_LEN]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"mylan-verification-salt-v1");
    hasher.update(key);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

/// Cabecera del paquete cifrado segun especificacion (48 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireHeader {
    pub version: u8,
    pub kdf: u8,
    pub cipher: u8,
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
}

impl WireHeader {
    pub fn new(salt: [u8; SALT_LEN], nonce: [u8; NONCE_LEN]) -> Self {
        Self {
            version: CURRENT_VERSION,
            kdf: KDF_ARGON2ID,
            cipher: CIPHER_XCHACHA20_POLY1305,
            salt,
            nonce,
        }
    }

    pub fn to_bytes(&self) -> [u8; HEADER_LEN] {
        let mut buf = [0u8; HEADER_LEN];
        buf[0..4].copy_from_slice(&MAGIC_BYTES);
        buf[4] = self.version;
        buf[5] = self.kdf;
        buf[6] = self.cipher;
        buf[7] = RESERVED_BYTE;
        buf[8..24].copy_from_slice(&self.salt);
        buf[24..48].copy_from_slice(&self.nonce);
        buf
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self, CryptoError> {
        if slice.len() < HEADER_LEN {
            return Err(CryptoError::HeaderTooShort(slice.len()));
        }

        let magic = [slice[0], slice[1], slice[2], slice[3]];
        if magic != MAGIC_BYTES {
            return Err(CryptoError::InvalidMagic(magic));
        }

        let version = slice[4];
        if version != CURRENT_VERSION {
            return Err(CryptoError::UnsupportedVersion(version));
        }

        let kdf = slice[5];
        if kdf != KDF_ARGON2ID {
            return Err(CryptoError::UnsupportedKdf(kdf));
        }

        let cipher = slice[6];
        if cipher != CIPHER_XCHACHA20_POLY1305 {
            return Err(CryptoError::UnsupportedCipher(cipher));
        }

        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&slice[8..24]);

        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&slice[24..48]);

        Ok(Self {
            version,
            kdf,
            cipher,
            salt,
            nonce,
        })
    }
}

/// Cifra un payload arbitrario usando XChaCha20-Poly1305 y antepone la cabecera fija de 48 bytes.
pub fn encrypt_package(
    key: &[u8; KEY_LEN],
    salt: &[u8; SALT_LEN],
    nonce: &[u8; NONCE_LEN],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let header = WireHeader::new(*salt, *nonce);
    let cipher = XChaCha20Poly1305::new(key.into());
    let xnonce = XNonce::from_slice(nonce);

    let ciphertext = cipher
        .encrypt(xnonce, plaintext)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;

    let mut package = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    package.extend_from_slice(&header.to_bytes());
    package.extend_from_slice(&ciphertext);

    Ok(package)
}

/// Cifra un payload generando salt y nonce aleatorios a partir de la contrasena maestra.
pub fn encrypt_with_passphrase(passphrase: &str, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let salt = random_bytes::<SALT_LEN>()?;
    let nonce = random_bytes::<NONCE_LEN>()?;
    let key = derive_key_argon2id(passphrase, &salt)?;

    encrypt_package(&key, &salt, &nonce, plaintext)
}

/// Descifra un paquete binario validando la cabecera e integridad criptografica con una clave directa de 32 bytes.
pub fn decrypt_package(key: &[u8; KEY_LEN], package: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let header = WireHeader::from_bytes(package)?;
    let ciphertext = &package[HEADER_LEN..];

    let cipher = XChaCha20Poly1305::new(key.into());
    let xnonce = XNonce::from_slice(&header.nonce);

    cipher
        .decrypt(xnonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// Descifra un paquete binario a partir de la contrasena maestra, derivando la clave con el salt de la cabecera.
pub fn decrypt_with_passphrase(passphrase: &str, package: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let header = WireHeader::from_bytes(package)?;
    let key = derive_key_argon2id(passphrase, &header.salt)?;

    decrypt_package(&key, package)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argon2id_derives_deterministic_key_with_same_salt() {
        let salt = [42u8; SALT_LEN];
        let key1 = derive_key_argon2id("mi-password-secreto", &salt).unwrap();
        let key2 = derive_key_argon2id("mi-password-secreto", &salt).unwrap();
        assert_eq!(*key1, *key2);

        let salt_diff = [43u8; SALT_LEN];
        let key3 = derive_key_argon2id("mi-password-secreto", &salt_diff).unwrap();
        assert_ne!(*key1, *key3);
    }

    #[test]
    fn test_mnemonic_generation_and_recovery() {
        let (phrase, key1) = generate_mnemonic_and_key().unwrap();
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 12, "Debe tener exactamente 12 palabras");

        let key2 = mnemonic_to_key(&phrase).unwrap();
        assert_eq!(
            *key1, *key2,
            "La clave derivada de las 12 palabras debe coincidir"
        );
    }

    #[test]
    fn test_key_verification_hash() {
        let key = [7u8; KEY_LEN];
        let hash1 = compute_key_verification_hash(&key);
        let hash2 = compute_key_verification_hash(&key);
        assert_eq!(hash1, hash2);

        let diff_key = [8u8; KEY_LEN];
        let hash3 = compute_key_verification_hash(&diff_key);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip_with_passphrase() {
        let secret_data = b"Hola Mundo desde MyLAN Zero-Knowledge Backup!".to_vec();
        let passphrase = "SuperClaveRobusta123!";

        let encrypted = encrypt_with_passphrase(passphrase, &secret_data).unwrap();
        assert!(encrypted.len() > HEADER_LEN + secret_data.len());
        assert_eq!(&encrypted[0..4], b"MYLN");

        let decrypted = decrypt_with_passphrase(passphrase, &encrypted).unwrap();
        assert_eq!(decrypted, secret_data);
    }

    #[test]
    fn test_decrypt_with_wrong_passphrase_fails() {
        let secret_data = b"Informacion confidencial de red".to_vec();
        let encrypted = encrypt_with_passphrase("correct-pass", &secret_data).unwrap();

        let result = decrypt_with_passphrase("wrong-pass", &encrypted);
        assert!(result.is_err());
        match result.unwrap_err() {
            CryptoError::DecryptionFailed => {}
            other => panic!("Esperaba DecryptionFailed, obtuve: {other:?}"),
        }
    }

    #[test]
    fn test_tampered_ciphertext_fails_integrity() {
        let secret_data = b"Datos integros".to_vec();
        let mut encrypted = encrypt_with_passphrase("pass123", &secret_data).unwrap();

        // Alteramos un byte del payload
        let last_idx = encrypted.len() - 1;
        encrypted[last_idx] ^= 0x55;

        let result = decrypt_with_passphrase("pass123", &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_header_fails() {
        let mut invalid_header = [0u8; 48];
        invalid_header[0..4].copy_from_slice(b"NOPE");
        let result = decrypt_package(&[0u8; 32], &invalid_header);
        assert!(result.is_err());
        match result.unwrap_err() {
            CryptoError::InvalidMagic(_) => {}
            other => panic!("Esperaba InvalidMagic, obtuve: {other:?}"),
        }
    }
}

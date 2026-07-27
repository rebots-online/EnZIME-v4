//! Optional payload encryption: Argon2id passphrase KDF + XChaCha20-Poly1305 AEAD.
//!
//! Encryption is the §13.1 `backup.encrypted` paid capability; plaintext
//! backup is always available per §13.5 ("never block export of user-created
//! data"). The cipher layout is documented inline; both the creator and the
//! restorer read exactly the same bytes the same way.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{aead::Aead, KeyInit, XChaCha20Poly1305, XNonce};

use crate::error::BackupError;
use crate::manifest::Argon2ParamsRecord;

/// Argon2id parameters used by default. Recorded in the manifest so a
/// future reader can rebuild the KDF even if the library default moves.
pub const DEFAULT_ARGON2_M_COST_KIB: u32 = 64 * 1024;
pub const DEFAULT_ARGON2_T_COST: u32 = 3;
pub const DEFAULT_ARGON2_P_COST: u32 = 4;
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 24;
pub const KEY_LEN: usize = 32;

/// A freshly-derived 32-byte key plus the salt/nonce/params used to derive
/// it; the latter three go into the manifest so the same KDF can be
/// reproduced on the restore side.
pub struct DerivedKey {
    pub key: [u8; KEY_LEN],
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
    pub params: Argon2ParamsRecord,
}

/// Derive a fresh key + nonce for an outbound encrypted backup. Uses
/// cryptographically-random salt and nonce.
pub fn derive_fresh(passphrase: &str) -> Result<DerivedKey, BackupError> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    fill_random(&mut salt)?;
    fill_random(&mut nonce)?;
    let params = Argon2ParamsRecord {
        m_cost_kib: DEFAULT_ARGON2_M_COST_KIB,
        t_cost: DEFAULT_ARGON2_T_COST,
        p_cost: DEFAULT_ARGON2_P_COST,
    };
    let key = kdf(passphrase, &salt, &params)?;
    Ok(DerivedKey {
        key,
        salt,
        nonce,
        params,
    })
}

/// Reproduce the same key from a recorded salt + params.
pub fn derive_from_record(
    passphrase: &str,
    salt: &[u8],
    params: &Argon2ParamsRecord,
) -> Result<[u8; KEY_LEN], BackupError> {
    if salt.len() != SALT_LEN {
        return Err(BackupError::FormatInvalid(format!(
            "kdf salt is {} bytes, expected {}",
            salt.len(),
            SALT_LEN
        )));
    }
    let mut salt_buf = [0u8; SALT_LEN];
    salt_buf.copy_from_slice(salt);
    kdf(passphrase, &salt_buf, params)
}

fn kdf(
    passphrase: &str,
    salt: &[u8; SALT_LEN],
    params: &Argon2ParamsRecord,
) -> Result<[u8; KEY_LEN], BackupError> {
    let params = Params::new(params.m_cost_kib, params.t_cost, params.p_cost, Some(KEY_LEN))
        .map_err(|e| BackupError::InvalidRequest(format!("argon2 params invalid: {e}")))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; KEY_LEN];
    argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|_| BackupError::PassphraseWrong)?;
    Ok(out)
}

/// Encrypt cleartext under a derived key. Output = ciphertext || Poly1305
/// tag (24 bytes of AAD-free XChaCha20-Poly1305 sealed message).
pub fn encrypt(key: &DerivedKey, cleartext: &[u8]) -> Result<Vec<u8>, BackupError> {
    let cipher = XChaCha20Poly1305::new(&key.key.into());
    let nonce = XNonce::from_slice(&key.nonce);
    cipher
        .encrypt(nonce, cleartext)
        .map_err(|_| BackupError::InvalidRequest("encryption failed".into()))
}

/// Decrypt ciphertext + tag produced by [`encrypt`]. Returns the original
/// cleartext or [`BackupError::PassphraseWrong`] for any authentication
/// failure.
pub fn decrypt(key: &[u8; KEY_LEN], nonce: &[u8; NONCE_LEN], ciphertext: &[u8]) -> Result<Vec<u8>, BackupError> {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XNonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| BackupError::PassphraseWrong)
}

fn fill_random(out: &mut [u8]) -> Result<(), BackupError> {
    use rand::RngCore;
    rand::thread_rng().fill_bytes(out);
    Ok(())
}

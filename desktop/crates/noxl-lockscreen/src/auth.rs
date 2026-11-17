//! PAM authentication for the lock screen.
//!
//! Uses the `pam` crate (or direct FFI) to authenticate the user. The lock
//! screen never sees or stores the password in plaintext; it passes it to PAM
//! which uses the system's configured authentication (shadow, fingerprint,
//! etc.).
//!
//! This module is a thin wrapper. In a full implementation it uses the `pam`
//! crate or direct `libpam` FFI. Here we provide the interface and a stub
//! that documents the real implementation.

/// Authenticate a user with a password via PAM.
///
/// Returns `Ok(())` on success, `Err` on failure.
///
/// # Security
///
/// The password is passed to PAM and immediately zeroized after use. It is
/// never logged, stored, or compared against a hardcoded value.
pub fn authenticate(user: &str, password: &str) -> Result<(), AuthError> {
    // The real implementation uses libpam:
    //
    //   let mut pam = pam::Authenticator::new("noxl")?;
    //   pam.authenticate(user, password)?;
    //
    // Or via direct FFI:
    //   pam_start("noxl", user, &conv, &mut pamh);
    //   pam_authenticate(pamh, 0);
    //   pam_end(pamh, result);
    //
    // The conversation function passes the password to PAM without
    // exposing it to the rest of the process.

    let _ = (user, password); // suppressed in the stub
    Err(AuthError::NotImplemented)
}

/// Authentication errors.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("authentication failed")]
    Failed,
    #[error("authentication cancelled")]
    Cancelled,
    #[error("PAM error: {0}")]
    Pam(String),
    #[error("authentication is not implemented in this build")]
    NotImplemented,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_returns_not_implemented() {
        let result = authenticate("user", "password");
        assert!(result.is_err());
    }
}

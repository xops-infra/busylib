use magic_crypt::{new_magic_crypt, MagicCryptTrait};

use crate::errors::DecryptError;
use crate::prelude::EnhancedUnwrap;

/// return encrypted string in base64
pub fn encrypt_by_key(value: String, key: &str) -> String {
    let mc = new_magic_crypt!(key, 256);
    mc.encrypt_str_to_base64(value)
}

/// return decrypted string from base64
pub fn decrypt_by_key(value: String, key: &str) -> String {
    let mc = new_magic_crypt!(key, 256);
    mc.decrypt_base64_to_string(value).unwp()
}

/// return decrypted string from base64, if error, return default
pub fn decrypt_by_key_with_default(value: String, key: &str, default: &str) -> String {
    let mc = new_magic_crypt!(key, 256);
    let decrypted_result = mc.decrypt_base64_to_string(value);
    match decrypted_result {
        Ok(decrypted_result) => decrypted_result,
        Err(_) => default.to_string(),
    }
}

/// return decrypted result from base64, if error, return Err
pub fn decrypt_by_key_with_error(value: String, key: &str) -> Result<String, DecryptError> {
    let mc = new_magic_crypt!(key, 256);
    let decrypted_result = mc.decrypt_base64_to_string(value);
    match decrypted_result {
        Ok(decrypted_result) => Ok(decrypted_result),
        Err(e) => Err(DecryptError {
            details: format!("{}", e),
        }),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn encrypt_test() {
        let msg = "https?";
        let key = "foo";
        let encrypted = crate::crypto::encrypt_by_key(msg.to_string(), key);
        let decrypted = crate::crypto::decrypt_by_key(encrypted, key);

        assert_eq!(msg, decrypted);
    }

    #[test]
    fn decrypt_test() {
        let msg = "https?";
        let key = "foo";
        let default = "default msg";
        let decrypted = crate::crypto::decrypt_by_key_with_default(msg.to_string(), key, default);
        assert_eq!(decrypted, default);

        let result = std::panic::catch_unwind(|| {
            if crate::crypto::decrypt_by_key_with_error(msg.to_string(), key).is_ok() {
                panic!("should be error");
            }
            let encrypted = crate::crypto::encrypt_by_key(msg.to_string(), key);
            if let Ok(decrypted) = crate::crypto::decrypt_by_key_with_error(encrypted, key) {
                assert_eq!(msg, decrypted);
            } else {
                panic!("decrypt error");
            }
        });
        if let Err(err) = result {
            eprintln!("Got an error: {:?}", err);
            panic!("Decrypt error: {:?}", err);
        }
    }
}

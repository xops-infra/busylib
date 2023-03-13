use magic_crypt::{new_magic_crypt, MagicCryptTrait};

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
}

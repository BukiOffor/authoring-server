use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct OtpData {
    pub code: String,
    pub expires_at: Instant,
    pub attempts: u8,
}

pub struct OtpManager {
    storage: Arc<RwLock<HashMap<String, OtpData>>>,
    expiration_time: Duration,
    max_attempts: u8,
}

impl OtpManager {
    pub fn new(expiration_minutes: u64, max_attempts: u8) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            expiration_time: Duration::from_secs(expiration_minutes * 60),
            max_attempts,
        }
    }

    pub fn generate_otp(&self, identifier: &str) -> String {
        let code = Self::generate_random_code(6);
        let expires_at = Instant::now() + self.expiration_time;

        let otp_data = OtpData {
            code: code.clone(),
            expires_at,
            attempts: 0,
        };

        let mut storage = self.storage.write().unwrap();
        storage.insert(identifier.to_string(), otp_data);

        code
    }

    pub fn verify_otp(&self, identifier: &str, code: &str) -> bool {
        let mut storage = self.storage.write().unwrap();

        if let Some(otp_data) = storage.get_mut(identifier) {
            // Check expiration
            if Instant::now() > otp_data.expires_at {
                storage.remove(identifier);
                return false;
            }

            // Check attempts
            if otp_data.attempts >= self.max_attempts {
                storage.remove(identifier);
                return false;
            }

            otp_data.attempts += 1;

            if otp_data.code == code {
                storage.remove(identifier);
                return true;
            }

            return false;
        }

        false
    }

    pub fn cleanup_expired(&self) {
        let mut storage = self.storage.write().unwrap();
        storage.retain(|_, otp_data| Instant::now() <= otp_data.expires_at);
    }

    fn generate_random_code(length: usize) -> String {
        let mut rng = rand::rng();
        (0..length)
            .map(|_| rng.random_range(0..10).to_string())
            .collect()
    }
}

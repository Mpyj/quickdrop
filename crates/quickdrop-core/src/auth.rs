//! تولید و اعتبارسنجی توکن و Pair Code

use rand::Rng;

/// طول توکن (به کاراکتر)
const TOKEN_LENGTH: usize = 32;

/// تولید یه توکن تصادفی hex
/// 
/// خروجی: رشته‌ای به طول ۳۲ کاراکتر از حروف a-f و اعداد 0-9
/// مثال: "a7b3c9d2e1f4..."
pub fn generate_token() -> String {
    let mut rng = rand::rng();
    (0..TOKEN_LENGTH)
        .map(|_| {
            let n: u8 = rng.random_range(0..16);
            char::from_digit(n as u32, 16).unwrap()
        })
        .collect()
}

/// تولید یه Pair Code ۴ رقمی
/// 
/// خروجی: رشته‌ای مثل "7429"
pub fn generate_pair_code() -> String {
    let mut rng = rand::rng();
    let code: u32 = rng.random_range(1000..10000);
    code.to_string()
}

/// چک می‌کنه که توکن داده شده با توکن اصلی مطابقت داره یا نه
pub fn verify_token(provided: &str, expected: &str) -> bool {
    // مقایسه‌ی constant-time برای جلوگیری از timing attack
    if provided.len() != expected.len() {
        return false;
    }
    provided
        .bytes()
        .zip(expected.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}
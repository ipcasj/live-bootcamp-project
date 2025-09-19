#[tokio::test]
async fn debug_password_hashing() {
    use argon2::{
        password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
        PasswordVerifier, Version,
    };

    // Test the hashing and verification logic directly
    let password = "password123";
    
    // Hash the password
    println!("Hashing password: {}", password);
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    );
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    let hash_string = password_hash.to_string();
    println!("Generated hash: {}", hash_string);
    
    // Verify the password
    println!("Verifying password...");
    let parsed_hash = PasswordHash::new(&hash_string).unwrap();
    let verification_result = argon2.verify_password(password.as_bytes(), &parsed_hash);
    println!("Direct verification result: {:?}", verification_result);
    
    // Test with wrong password
    println!("Testing with wrong password...");
    let wrong_verification = argon2.verify_password("wrongpassword".as_bytes(), &parsed_hash);
    println!("Wrong password verification result: {:?}", wrong_verification);
}
// Goose load test usage examples:
//
// Run against local server:
// cargo run --bin load_test -- --host http://localhost:3000
//
// Run against deployed server:
// cargo run --bin load_test -- --host http://your-server-ip:3000
//
// Additional Goose options:
//   --users 50         # Number of concurrent users
//   --hatch-rate 5     # How fast users are started per second
//   --run-time 30s     # How long to run the test
//
// Example with options:
// cargo run --bin load_test -- --host http://localhost:3000 --users 50 --hatch-rate 5 --run-time 30s

use goose::prelude::*;

async fn signup_user(user: &mut GooseUser) -> TransactionResult {
    let _response = user.post("/signup", r#"{"email":"loadtest@example.com","password":"password"}"#).await?;
    Ok(())
}

async fn login_user(user: &mut GooseUser) -> TransactionResult {
    let _response = user.post("/login", r#"{"email":"loadtest@example.com","password":"password"}"#).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("SignupUser").register_transaction(transaction!(signup_user))
        )
        .register_scenario(
            scenario!("LoginUser").register_transaction(transaction!(login_user))
        )
        .execute()
        .await?;
    Ok(())
}

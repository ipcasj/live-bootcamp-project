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

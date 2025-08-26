// The User struct should contain 3 fields. email, which is a String; 
// password, which is also a String; and requires_2fa, which is a boolean. 
//Note: You will also need to update the User struct to derive a few traits for the unit tests to pass.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct User {
    pub email: String,
    pub password: String,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: String, password: String, requires_2fa: bool) -> Self {
        User {
            email,
            password,
            requires_2fa,
        }
    }
}
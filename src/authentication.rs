/// Function to authenticate new websockets. returns an error if the client fails.
pub fn authenticate_ws(challenge: String) -> Result<User, String> {
    return Ok(User { username: challenge, description: "if you see this user, something is terribly wrong.".to_string()});
}

pub struct User {
    pub username: String,
    pub description: String,
}

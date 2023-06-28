use serde::Serialize;

/// The (public) id of the User.
#[derive(Debug, Serialize)]
pub struct UserId(i64);

impl UserId {
    pub fn as_value(&self) -> i64 {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl From<i64> for UserId {
    fn from(id: i64) -> Self {
        UserId(id)
    }
}

/// The main representation of the User. <br/>
/// It contains most of the details (except for password).
#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub bio: String,
    pub image: Option<String>,
}

/// It includes all user attributes that are persisted in the database.
pub struct UserEntry {
    pub user: User,
    pub password: String,
    pub salt: String,
}

impl Into<User> for UserEntry {
    fn into(self) -> User {
        self.user
    }
}

/// A common representation of a `User`, used in multiple use cases.
#[derive(Clone, Debug, Serialize)]
pub struct UserProfile {
    #[serde(skip_serializing)]
    pub user_id: i64,
    pub username: String,
    pub bio: String,
    pub image: Option<String>,
    pub following: bool,
}

impl UserProfile {
    pub fn new_basic(user_id: i64) -> Self {
        Self {
            user_id,
            username: "".into(),
            bio: "".into(),
            image: None,
            following: false,
        }
    }
}

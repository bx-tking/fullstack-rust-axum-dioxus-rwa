use std::sync::Arc;

use sqlx::{postgres::PgRow, FromRow, Row};

use crate::{
    db::DbConnPool,
    domain::model::{User, UserEntry, UserId},
    handlers::UserProfileDTO,
    AppError, AppUseCase,
};

/// A Postgres specific implementation of `UserRepo`.
pub struct UserRepo {
    dbcp: Arc<DbConnPool>,
}

impl UserRepo {
    pub fn new(dbcp: Arc<DbConnPool>) -> Self {
        Self { dbcp }
    }

    pub async fn save(&self, user: &User, pwd: String, salt: String) -> Result<i64, AppError> {
        match sqlx::query(
            "INSERT INTO accounts(email, username, password, salt) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(&user.email)
        .bind(&user.username)
        .bind(pwd)
        .bind(salt)
        .fetch_one(self.dbcp.as_ref())
        .await
        {
            Ok(row) => Ok(row.get("id")),
            Err(err) => Err(AppError::from((err, AppUseCase::UserRegister))),
        }
    }

    pub async fn get_by_email(
        &self,
        email: &String,
        usecase: AppUseCase,
    ) -> Result<UserEntry, AppError> {
        let entry = sqlx::query_as::<_, UserEntry>(
            "SELECT id, email, username, password, salt, bio, image FROM accounts WHERE email = $1",
        )
        .bind(&email)
        .fetch_one(self.dbcp.as_ref())
        .await;
        match entry {
            Ok(entry) => Ok(entry),
            Err(err) => Err(AppError::from((err, usecase))),
        }
    }

    pub async fn get_by_id(&self, id: &UserId, usecase: AppUseCase) -> Result<UserEntry, AppError> {
        let entry = sqlx::query_as::<_, UserEntry>(
            "SELECT email, username, password, salt, bio, image FROM accounts WHERE id = $1",
        )
        .bind(id.as_value())
        .fetch_one(self.dbcp.as_ref())
        .await;
        match entry {
            Ok(entry) => Ok(entry),
            Err(err) => Err(AppError::from((err, usecase))),
        }
    }

    pub async fn get_profile_by_username(
        &self,
        username: &String,
        usecase: AppUseCase,
    ) -> Result<UserProfileDTO, AppError> {
        let mut user_id = 0_i64;
        let res = sqlx::query("SELECT id, bio, image FROM accounts WHERE username = $1")
            .bind(username)
            .map(|row: PgRow| {
                user_id = row.get("id");
                UserProfileDTO {
                    username: username.clone(),
                    bio: row.get("bio"),
                    image: row.get("image"),
                    following: None,
                }
            })
            .fetch_one(self.dbcp.as_ref())
            .await;
        match res {
            Ok(mut dto) => {
                let followings = self.get_followings(user_id).await;
                dto.following = followings.ok();
                Ok(dto)
            }
            Err(err) => Err(AppError::from((err, usecase))),
        }
    }

    async fn get_followings(&self, user_id: i64) -> Result<Vec<UserId>, AppError> {
        let result = sqlx::query("SELECT followed_user_id FROM followings WHERE user_id = $1")
            .bind(user_id)
            .map(|row: PgRow| UserId::from(row.get::<i64, _>("followed_user_id")))
            .fetch_all(self.dbcp.as_ref())
            .await?;
        Ok(result)
    }

    pub async fn update_by_id(
        &self,
        id: UserId,
        email: Option<String>,
        bio: Option<String>,
        image: Option<String>,
    ) -> Result<UserEntry, AppError> {
        if email.is_none() && bio.is_none() && image.is_none() {
            return Err(AppError::InvalidInput);
        }
        match self.get_by_id(&id, AppUseCase::UpdateUser).await {
            Ok(mut entry) => {
                entry.user.email = email.unwrap_or_else(|| entry.user.email);
                entry.user.bio = bio.unwrap_or_else(|| entry.user.bio);
                entry.user.image = if image.is_some() {
                    image
                } else {
                    entry.user.image
                };
                match sqlx::query(
                    "UPDATE accounts SET email = $1, bio = $2, image = $3 WHERE id = $4",
                )
                .bind(&entry.user.email)
                .bind(&entry.user.bio)
                .bind(&entry.user.image)
                .bind(id.as_value())
                .execute(self.dbcp.as_ref())
                .await
                {
                    Ok(_) => Ok(entry),
                    Err(err) => Err(AppError::from((err, AppUseCase::UpdateUser))),
                }
            }
            Err(err) => Err(err),
        }
    }
}

// ---------------------------------------
//    From<sqlx::Error> implementations
// ---------------------------------------

impl FromRow<'_, PgRow> for User {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            email: row.get("email"),
            username: row.get("username"),
            bio: row.get("bio"),
            image: row.get("image"),
        })
    }
}

impl FromRow<'_, PgRow> for UserEntry {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            user: User {
                id: row.try_get("id").unwrap_or_default(),
                email: row.get("email"),
                username: row.get("username"),
                bio: row.get("bio"),
                image: row.try_get("image").unwrap_or_default(),
            },
            password: row.get("password"),
            salt: row.get("salt"),
        })
    }
}

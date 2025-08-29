use crate::helpers::dto::auth::{AuthBodyDto, JwtPayloadDto};
use crate::helpers::dto::tasks::TaskMigrationDto;
use crate::helpers::password_hasher;
use crate::models::tasks::Tasks;
use crate::models::topic::Topic;
use crate::models::user::UserDto;
use crate::{DbConn, schema};
use crate::{DbPool, error::ModuleError, helpers::dto::auth::AuthPayloadDto, models::user::User};
use diesel::Connection;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use reqwest::Client;
use std::env;
use std::sync::Arc;
use uuid::Uuid;

pub async fn authenticate_user(
    payload: AuthPayloadDto,
    pool: Arc<DbPool>,
) -> Result<JwtPayloadDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    let user: Option<User> = schema::user::table
        .filter(schema::user::email.eq(payload.email.clone()))
        .select(User::as_select())
        .first(&mut conn)
        .optional()
        .map_err(|e| {
            ModuleError::InternalError(format!("Error fetching item from table: {}", e))
        })?;
    if let Some(user) = user {
        let is_password = crate::helpers::password_verfier(&payload.password, &user.password_hash);
        if !is_password {
            return Err(ModuleError::WrongCredentials);
        }
        let id: Uuid = Uuid::parse_str(&user.id).map_err(|e| ModuleError::Error(e.to_string()))?;
        return Ok(JwtPayloadDto::new(id));
    } else {
        let url = env::var("UPSTREAM_SERVER")
            .map(|server| format!("{}/auth", server))
            .map_err(|e| ModuleError::Error(e.to_string()))?;

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ModuleError::InternalError(e.to_string()))?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| ModuleError::Error(e.to_string()))?;
            let auth_response: AuthBodyDto = serde_json::from_str(&body)
                .map_err(|e| ModuleError::InternalError(e.to_string()))?;

            let url = env::var("UPSTREAM_SERVER")
                .map(|server| format!("{}/users/fetch/{}", server, auth_response.id))
                .map_err(|e| ModuleError::Error(e.to_string()))?;

            let result = client
                .get(url)
                .bearer_auth(auth_response.access_token.clone())
                .send()
                .await
                .map_err(|e| ModuleError::InternalError(e.to_string()))?;
            if !result.status().is_success() {
                return Err(ModuleError::InternalError(
                    "Something Went Wrong, please contact Adminstartor".into(),
                ));
            }
            let body = result
                .text()
                .await
                .map_err(|e| ModuleError::Error(e.to_string()))?;
            let dto: UserDto = serde_json::from_str(&body)
                .map_err(|e| ModuleError::InternalError(e.to_string()))?;
            let id = dto.id.clone();
            let password = password_hasher(payload.password.clone())?;
            let user = User::from_dto(dto, password, auth_response.refresh_token);
            populate_table(user,&mut conn, &client, auth_response.access_token.clone()).await?;

            Ok(JwtPayloadDto::new(id))
        } else {
            return Err(ModuleError::Error(
                "Upstream Server is not active yet, please contact Adminstrator".into(),
            ));
        }
    }
}

// get the topic and everything

// get the topic details about the topic asignned

// from the topic details, if the topic is a subtopic fetch its parent topic;

pub async fn populate_table(
    user:User,
    conn: &mut DbConn,
    client: &Client,
    access_token: String,
) -> Result<(), ModuleError> {
    let url = env::var("UPSTREAM_SERVER")
        .map(|server| format!("{}/tasks/migration/author", server))
        .map_err(|e| ModuleError::Error(e.to_string()))?;

    let result = client
        .get(url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    if !result.status().is_success() {
        return Err(ModuleError::InternalError(
            "Something Went Wrong, please contact Adminstartor".into(),
        ));
    }
    println!("broke here");
    let body = result
        .text()
        .await
        .map_err(|e| ModuleError::Error(e.to_string()))?;

    let task: TaskMigrationDto =
        serde_json::from_str(&body).map_err(|e| ModuleError::InternalError(e.to_string()))?;

    // for t in task.tasks {
    //     let task = Tasks::from(t);
    //     diesel::insert_into(schema::tasks::table)
    //         .values(&task)
    //         .on_conflict_do_nothing()
    //         .execute(conn)
    //         .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    // }
    //  for t in task.topics {
    //     let topic = Topic::from(t);
    //     diesel::insert_into(schema::topics::table)
    //         .values(&topic)
    //         .on_conflict_do_nothing()
    //         .execute(conn)
    //         .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    // }

    conn.transaction::<_, ModuleError, _>(|conn| {
        diesel::insert_into(schema::user::table)
                .values(user)
                .execute(conn)
                .map_err(|e| ModuleError::InternalError(e.to_string()))?;
        for t in task.tasks {
            let task = Tasks::from(t);
            diesel::insert_into(schema::tasks::table)
                .values(&task)
                .on_conflict_do_nothing()
                .execute(conn)
                .map_err(|e| ModuleError::InternalError(e.to_string()))?;
        }
        for t in task.topics {
            let topic = Topic::from(t);
            diesel::insert_into(schema::topics::table)
                .values(&topic)
                .on_conflict_do_nothing()
                .execute(conn)
                .map_err(|e| ModuleError::InternalError(e.to_string()))?;
        }
        Ok(())
    })?;
    Ok(())
}

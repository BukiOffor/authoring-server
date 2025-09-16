use crate::helpers::dto::auth::{JwtPayloadDto, LoginResponse};
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
use crate::helpers::dto::MessageDto;

pub async fn authenticate_user(
    payload: AuthPayloadDto,
    pool: Arc<DbPool>,
) -> Result<JwtPayloadDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    let client = Client::builder()
        .cookie_store(true)
        .build()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    let user: Option<User> = schema::user::table
        .filter(schema::user::email.eq(payload.email.clone()))
        .select(User::as_select())
        .first(&mut conn)
        .optional()
        .map_err(|e| {
            ModuleError::InternalError(format!("Error fetching item from table: {}", e))
        })?;
    if let Some(mut user) = user {
        let is_password = crate::helpers::password_verfier(&payload.password, &user.password_hash);
        if !is_password {
            return Err(ModuleError::WrongCredentials);
        }
        if let Ok(_) = try_login(payload.clone(), &client).await {
            user.bearer_token = String::new();
            // ❌❌❌ User object does not update
            populate_table(user.clone(), &mut conn, &client).await?;
        }
        return Ok(JwtPayloadDto::new(user.id));
    } else {
        let url = env::var("UPSTREAM_SERVER")
            .map(|server| format!("{}/auth", server))
            .map_err(|e| ModuleError::Error(e.to_string()))?;

        //let client = reqwest::Client::new();
        let response = client.post(url).json(&payload).send().await.map_err(|_| {
            ModuleError::InternalError(
                "Upstream Server is not active yet, please contact Adminstrator".into(),
            )
        })?;

        if response.status().is_success() {
            let body = response
                .text()
                .await
                .map_err(|e| ModuleError::Error(e.to_string()))?;
            let auth_response: LoginResponse = serde_json::from_str(&body)
                .map_err(|e| ModuleError::InternalError(e.to_string()))?;

            let url = env::var("UPSTREAM_SERVER")
                .map(|server| format!("{}/users/fetch/{}", server, auth_response.id))
                .map_err(|e| ModuleError::Error(e.to_string()))?;

            let result = client
                .get(url)
                //.bearer_auth(auth_response.access_token.clone())
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
            let id = dto.id.clone().into();
            let password = password_hasher(payload.password.clone())?;
            let user = User::from_dto(dto, password);
            populate_table(user, &mut conn, &client).await?;

            Ok(JwtPayloadDto::new(id))
        } else {
            return Err(ModuleError::Error(
                "Upstream Server is not active yet, please contact Adminstrator".into(),
            ));
        }
    }
}

pub async fn populate_table(
    user: User,
    conn: &mut DbConn,
    client: &Client,
) -> Result<(), ModuleError> {
    let url = env::var("UPSTREAM_SERVER")
        .map(|server| format!("{}/tasks/migration/author", server))
        .map_err(|e| ModuleError::Error(e.to_string()))?;

    let result = client
        .get(url)
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

    let task: TaskMigrationDto =
        serde_json::from_str(&body).map_err(|e| ModuleError::InternalError(e.to_string()))?;

    conn.transaction::<_, ModuleError, _>(|conn| {
        diesel::insert_into(schema::user::table)
            .values(&user)
            .on_conflict(schema::user::id)
            .do_update()
            .set(schema::user::bearer_token.eq(&user.bearer_token))
            .execute(conn)
            .map_err(|e| ModuleError::InternalError(e.to_string()))?;

        for t in task.tasks {
            let task = Tasks::from(t);
            diesel::insert_into(schema::tasks::table)
                .values(&task)
                .on_conflict((schema::tasks::task_id, schema::tasks::topic_id))
                .do_update()
                .set((
                    schema::tasks::num_of_questions.eq(&task.num_of_questions),
                    schema::tasks::due_date.eq(&task.due_date),
                ))
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

pub async fn try_login(
    payload: AuthPayloadDto,
    client: &Client,
) -> Result<LoginResponse, ModuleError> {
    let url = env::var("UPSTREAM_SERVER")
        .map(|server| format!("{}/auth", server))
        .map_err(|e| ModuleError::Error(e.to_string()))?;

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
        let auth_response: LoginResponse =
            serde_json::from_str(&body).map_err(|e| ModuleError::InternalError(e.to_string()))?;

        Ok(auth_response)
    } else if response.status().is_client_error() {
        // parse body into server error type
        let message: MessageDto = serde_json::from_str(&response.text().await.unwrap_or_default())
            .map_err(|_| {
                ModuleError::InternalError(
                    "Could not deserialize error message from upstream server".into(),
                )
            })?;
        return Err(ModuleError::InternalError(
            format!("Upstream: {}",message.message).into(),
        ));
    } else {
        Err(ModuleError::InternalError(
            "Upstream server is offline".into(),
        ))
    }
}

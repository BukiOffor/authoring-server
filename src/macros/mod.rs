pub mod db;

#[macro_export]
macro_rules! log_route {
    ($method:ident, $path:expr, $handler:expr) => {{
        tracing::info!(
            "Registered: {} {}",
            stringify!($method).to_uppercase(),
            $path
        );
        axum::routing::$method($handler)
    }};
}

#[macro_export]
macro_rules! guard {
    ($key:expr, $permission:ident, $pool:expr ) => {{
        use crate::dto::auth::RedisUser;
        use redis::AsyncCommands;
        let key: String = $key.into();
        let mut conn = $pool
            .get()
            .await
            .map_err(|e| ModuleError::InternalError(e.to_string()))?;
        let response: String = conn
            .get(key)
            .await
            .map_err(|_| ModuleError::InternalError("Could not get User from Redis".into()))?;
        let redis_user: RedisUser = serde_json::from_str(&response).map_err(|_| {
            ModuleError::InternalError("Could not deserialze string into Redis User".into())
        })?;
        match redis_user
            .permissions
            .iter()
            .any(|permissions| $permission.eq(permissions))
        {
            true => (),
            false => return Err(ModuleError::PermissionDenied),
        }
    }};
}

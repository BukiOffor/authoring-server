use std::sync::Arc;

use diesel::RunQueryDsl;

use crate::{DbPool, error::ModuleError, helpers::dto::subject::ItemReadyStats};

pub fn get_item_count_for_publishing(
    subject_id: &str,
    task_id: &str,
    pool: Arc<DbPool>,
) -> Result<Vec<ItemReadyStats>, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let query = diesel::sql_query(crate::helpers::querys::GET_SUBJECT_ITEM_READY_DETAILS);
    let stats: Vec<ItemReadyStats> = query
        .bind::<diesel::sql_types::Text, _>(subject_id)
        .bind::<diesel::sql_types::Text, _>(task_id)
        .load(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    Ok(stats)
}

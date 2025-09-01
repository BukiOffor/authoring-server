use diesel::{prelude::*, sql_types::Text};
//use uuid::Uuid;

use std::sync::Arc;

use crate::{
    DbPool,
    error::ModuleError,
    helpers::{
        self,
        dto::{
            MessageDto,
            items::{ItemStats, PassageWithItems},
        },
    },
    insert,
    models::{item::Items, item_options::ItemOptions},
    schema::{item_options, items},
};

pub fn create_item(
    pool: Arc<DbPool>,
    item: Items,
    options: Vec<ItemOptions>,
) -> Result<MessageDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    conn.transaction::<_, ModuleError, _>(|conn| {
        insert!(items::table, item, conn);
        insert!(item_options::table, options, conn);
        Ok(())
    })?;

    Ok(MessageDto {
        message: "Item has been created successfully".to_string(),
    })
}

pub fn create_passage_and_items(
    pool: Arc<DbPool>,
    payload: PassageWithItems,
) -> Result<MessageDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let passage = payload.passage;
    conn.transaction::<_, ModuleError, _>(|conn| {
        insert!(crate::schema::passages::table, passage, conn);

        for item_payload in payload.items {
            let item = item_payload.item;
            insert!(items::table, item, conn);
            for option in item_payload.options {
                insert!(item_options::table, option, conn);
            }
        }

        Ok(())
    })?;
    Ok("Items has been created successfully".into())
}

pub fn fetch_item_stats(subject_id: &str, pool: Arc<DbPool>) -> Result<ItemStats, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let task_id = helpers::get_current_user_task(&mut conn)?.unwrap_or_default();
    let query = diesel::sql_query(
        r#"
        SELECT
            COUNT(*) AS total_items,
            COUNT(*) FILTER (WHERE i.status = 'Draft') AS total_drafts,
            COUNT(*) FILTER (WHERE i.status = 'Ready') AS total_ready,
            COUNT(*) FILTER (WHERE i.status = 'Published') AS total_published
        FROM items i
        WHERE i.subject_id = $1
        AND i.task_id = $2
    "#,
    )
    .bind::<Text, _>(subject_id)
    .bind::<Text, _>(task_id);

    let stats: ItemStats = query
        .get_result(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    Ok(stats)
}

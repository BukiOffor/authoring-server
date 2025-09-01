use diesel::prelude::*;
//use uuid::Uuid;

use std::sync::Arc;

use crate::{
    DbPool,
    error::ModuleError,
    helpers::dto::MessageDto,
    insert,
    models::{item::Items, item_options::ItemOptions, passages::Passage},
    schema::{item_options, items},
};

pub fn create_item(
    pool: Arc<DbPool>,
    item: Vec<Items>,
    options: Vec<ItemOptions>,
    passages: Vec<Passage>,
) -> Result<MessageDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    conn.transaction::<_, ModuleError, _>(|conn| {
        insert!(items::table, item, conn);
        insert!(item_options::table, options, conn);
        insert!(crate::schema::passages::table, passages, conn);
        Ok(())
    })?;

    Ok(MessageDto {
        message: "Items has been created successfully".to_string(),
    })
}

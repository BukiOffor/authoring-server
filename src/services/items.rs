use crate::models::item::ItemStatus;
use crate::{
    DbPool,
    error::ModuleError,
    helpers::{
        self,
        dto::{
            MessageDto,
            items::{EditItemDto, ItemStats, PassageWithItems, display::*},
        },
    },
    insert,
    models::{item::Items, item_options::ItemOptions, passages::Passage},
    schema::{item_options, items, passages, topics},
};
use diesel::{prelude::*, sql_types::Text};
use std::collections::HashMap;
use std::sync::Arc;

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
// if item status is ready, validate that all the options are avail and correct
pub fn update_item_status(
    item_id: String,
    status: ItemStatus,
    pool: Arc<DbPool>,
) -> Result<MessageDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    diesel::update(items::table.filter(items::id.eq(item_id.clone())))
        .set((
            items::status.eq(status),
            items::updated_at.eq(chrono::Local::now().naive_local()),
        ))
        .execute(&mut conn)
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    Ok("Item has been updated successfully".into())
}

pub fn update_item(
    item_id: String,
    dto: EditItemDto,
    pool: Arc<DbPool>,
    publish: bool,
) -> Result<MessageDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    conn.transaction::<_, ModuleError, _>(
        |conn: &mut diesel::r2d2::PooledConnection<
            diesel::r2d2::ConnectionManager<SqliteConnection>,
        >| {
            let status = if publish {
                ItemStatus::Ready
            } else {
                ItemStatus::Draft
            };

            let item = dto.item;
            let options = dto.options;
            diesel::update(items::table.filter(items::id.eq(item_id.clone())))
                .set((
                    items::title.eq(item.title),
                    items::text.eq(item.text),
                    items::difficulty.eq(item.difficulty),
                    items::taxonomy.eq(item.taxonomy),
                    items::status.eq(status),
                    items::updated_at.eq(chrono::Local::now().naive_local()),
                ))
                .execute(conn)
                .map_err(|e| ModuleError::InternalError(e.to_string()))?;

            for option in options {
                diesel::update(item_options::table.filter(item_options::id.eq(option.id)))
                    .set((
                        item_options::label.eq(option.label),
                        item_options::value.eq(option.value),
                        item_options::is_answer.eq(option.is_answer),
                    ))
                    .execute(conn)
                    .map_err(|e| ModuleError::InternalError(e.to_string()))?;
            }
            Ok(())
        },
    )?;
    Ok("Item has been updated successfully".into())
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
            COUNT(*) FILTER (WHERE i.status = '"Draft"') AS total_drafts,
            COUNT(*) FILTER (WHERE i.status = '"Ready"') AS total_ready,
            COUNT(*) FILTER (WHERE i.status = '"Submitted"') AS total_published
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

pub fn delete_item(item_id: String, pool: Arc<DbPool>) -> Result<MessageDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;
    conn.transaction::<_, ModuleError, _>(|conn| {
        diesel::delete(items::table.filter(items::id.eq(item_id.clone()))).execute(conn)?;
        diesel::delete(item_options::table.filter(item_options::item_id.eq(item_id)))
            .execute(conn)?;
        Ok(())
    })?;
    Ok("Item has been deleted successfully".into())
}

pub fn fetch_topic_items_with_subtopics(
    parent_topic_id: &str,
    task_id: &str,
    status: ItemStatus,
    pool: Arc<DbPool>,
) -> Result<TopicItemsDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    // Find all relevant topic IDs (parent and direct children) ---
    let subtopic_ids: Vec<String> = topics::table
        .filter(topics::parent_topic_id.eq(parent_topic_id))
        .select(topics::id)
        .load(&mut conn)?;

    let mut relevant_topic_ids = subtopic_ids.clone();
    relevant_topic_ids.push(parent_topic_id.to_string());

    // --- Step 2: Fetch all relevant models from the database ---
    let relevant_items = items::table
        .filter(items::topic_id.eq_any(&relevant_topic_ids))
        .filter(items::task_id.eq(task_id))
        .filter(items::status.eq(status))
        .select(Items::as_select())
        .load::<Items>(&mut conn)?;

    let topic_map: HashMap<String, String> = topics::table
        .filter(topics::id.eq_any(&relevant_topic_ids))
        .select((topics::id, topics::name))
        .load::<(String, String)>(&mut conn)?
        .into_iter()
        .collect();

    let all_options = crate::schema::item_options::table
        .filter(crate::schema::item_options::item_id.eq_any(relevant_items.iter().map(|i| &i.id)))
        .select(ItemOptions::as_select())
        .load::<ItemOptions>(&mut conn)?;

    // Fetch all necessary passages
    let passage_ids: Vec<String> = relevant_items
        .iter()
        .filter_map(|i| i.passage_id.as_ref())
        .cloned()
        .collect();
    let passage_map: HashMap<String, Passage> = if !passage_ids.is_empty() {
        passages::table
            .filter(passages::id.eq_any(&passage_ids))
            .select(Passage::as_select())
            .load::<Passage>(&mut conn)?
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect()
    } else {
        HashMap::new()
    };

    // Group options by item ID for efficient mapping ---
    let mut options_by_item: HashMap<String, Vec<ItemOptions>> = HashMap::new();
    for opt in all_options {
        options_by_item
            .entry(opt.item_id.clone())
            .or_default()
            .push(opt);
    }

    // Process and organize all fetched items into DTOs ---
    // Temporary structure to hold processed data before final assembly
    struct TopicData {
        standalone_items: Vec<ItemDto>,
        passages: HashMap<String, PassageViewDto>,
    }

    let mut data_by_topic: HashMap<String, TopicData> = HashMap::new();

    for item in relevant_items {
        let options_dto = options_by_item
            .remove(&item.id)
            .unwrap_or_default()
            .into_iter()
            .map(|opt| OptionDto {
                id: opt.id,
                label: opt.label,
                value: opt.value,
                is_answer: opt.is_answer,
            })
            .collect();

        let item_dto = ItemDto {
            id: item.id,
            title: item.title,
            text: item.text,
            question_type: format!("{:?}", item.question_type),
            difficulty: item.difficulty,
            status: format!("{:?}", item.status),
            options: options_dto,
        };

        // Get the topic data entry for the current item's topic
        let topic_data = data_by_topic
            .entry(item.topic_id)
            .or_insert_with(|| TopicData {
                standalone_items: Vec::new(),
                passages: HashMap::new(),
            });

        // Add the item to either a passage or the standalone list
        if let Some(passage_id) = &item.passage_id {
            if let Some(passage_model) = passage_map.get(passage_id) {
                let passage_dto = topic_data
                    .passages
                    .entry(passage_id.clone())
                    .or_insert_with(|| PassageViewDto {
                        id: passage_model.id.clone(),
                        rubric: passage_model.rubric.clone().unwrap_or_default(),
                        stem: passage_model.stem.clone(),
                        items: Vec::new(),
                    });
                passage_dto.items.push(item_dto);
            }
        } else {
            topic_data.standalone_items.push(item_dto);
        }
    }

    // Assemble the final response DTO ---
    let root_data = data_by_topic
        .remove(parent_topic_id)
        .unwrap_or_else(|| TopicData {
            standalone_items: Vec::new(),
            passages: HashMap::new(),
        });

    let root_items_and_passages = ItemsAndPassages {
        items: root_data.standalone_items,
        passages: root_data.passages.into_values().collect(),
    };

    let subtopics_dto = subtopic_ids
        .into_iter()
        .filter_map(|sub_id| {
            data_by_topic.remove(&sub_id).map(|data| {
                let items_and_passages = ItemsAndPassages {
                    items: data.standalone_items,
                    passages: data.passages.into_values().collect(),
                };
                SubtopicDto {
                    id: sub_id.clone(),
                    name: topic_map.get(&sub_id).cloned().unwrap_or_default(),
                    // The DTO expects a Vec, so we wrap it
                    items: vec![items_and_passages],
                }
            })
        })
        .collect();

    Ok(TopicItemsDto {
        topic_id: parent_topic_id.to_string(),
        task_id: task_id.to_string(),
        items: root_items_and_passages,
        subtopics: subtopics_dto,
    })
}

// pub fn fetch_topic_items_with_subtopics(
//     parent_topic_id: &str,
//     task_id: &str,

//     pool: Arc<DbPool>,
// ) -> Result<TopicItemsDto, ModuleError> {
//     let mut conn = pool
//         .get()
//         .map_err(|e| ModuleError::InternalError(e.to_string()))?;

//     // Find all relevant topic IDs (parent and direct children) ---
//     let subtopic_ids: Vec<String> = topics::table
//         .filter(topics::parent_topic_id.eq(parent_topic_id))
//         .select(topics::id)
//         .load(&mut conn)?;

//     let mut relevant_topic_ids = subtopic_ids.clone();
//     relevant_topic_ids.push(parent_topic_id.to_string());

//     // --- Step 2: Fetch all relevant models from the database ---
//     let relevant_items = items::table
//         .filter(items::topic_id.eq_any(&relevant_topic_ids))
//         .filter(items::task_id.eq(task_id))
//         .select(Items::as_select())
//         .load::<Items>(&mut conn)?;

//     let topic_map: HashMap<String, String> = topics::table
//         .filter(topics::id.eq_any(&relevant_topic_ids))
//         .select((topics::id, topics::name))
//         .load::<(String, String)>(&mut conn)?
//         .into_iter()
//         .collect();

//     let all_options = crate::schema::item_options::table
//         .filter(crate::schema::item_options::item_id.eq_any(relevant_items.iter().map(|i| &i.id)))
//         .select(ItemOptions::as_select())
//         .load::<ItemOptions>(&mut conn)?;

//     // Fetch all necessary passages
//     let passage_ids: Vec<String> = relevant_items
//         .iter()
//         .filter_map(|i| i.passage_id.as_ref())
//         .cloned()
//         .collect();
//     let passage_map: HashMap<String, Passage> = if !passage_ids.is_empty() {
//         passages::table
//             .filter(passages::id.eq_any(&passage_ids))
//             .select(Passage::as_select())
//             .load::<Passage>(&mut conn)?
//             .into_iter()
//             .map(|p| (p.id.clone(), p))
//             .collect()
//     } else {
//         HashMap::new()
//     };

//     // --- Step 3: Group options by item ID for efficient mapping ---
//     let mut options_by_item: HashMap<String, Vec<ItemOptions>> = HashMap::new();
//     for opt in all_options {
//         options_by_item
//             .entry(opt.item_id.clone())
//             .or_default()
//             .push(opt);
//     }

//     // --- Step 4: Process and organize all fetched items into DTOs ---
//     // Temporary structure to hold processed data before final assembly
//     struct TopicData {
//         standalone_items: Vec<ItemDto>,
//         passages: HashMap<String, PassageViewDto>,
//     }

//     let mut data_by_topic: HashMap<String, TopicData> = HashMap::new();

//     for item in relevant_items {
//         let options_dto = options_by_item
//             .remove(&item.id)
//             .unwrap_or_default()
//             .into_iter()
//             .map(|opt| OptionDto {
//                 id: opt.id,
//                 label: opt.label,
//                 value: opt.value,
//                 is_answer: opt.is_answer,
//             })
//             .collect();

//         let item_dto = ItemDto {
//             id: item.id,
//             title: item.title,
//             text: item.text,
//             question_type: format!("{:?}", item.question_type),
//             difficulty: item.difficulty,
//             status: format!("{:?}", item.status),
//             options: options_dto,
//         };

//         // Get the topic data entry for the current item's topic
//         let topic_data = data_by_topic
//             .entry(item.topic_id)
//             .or_insert_with(|| TopicData {
//                 standalone_items: Vec::new(),
//                 passages: HashMap::new(),
//             });

//         // Add the item to either a passage or the standalone list
//         if let Some(passage_id) = &item.passage_id {
//             if let Some(passage_model) = passage_map.get(passage_id) {
//                 let passage_dto = topic_data
//                     .passages
//                     .entry(passage_id.clone())
//                     .or_insert_with(|| PassageViewDto {
//                         id: passage_model.id.clone(),
//                         stem: passage_model.stem.clone(),
//                         items: Vec::new(),
//                     });
//                 passage_dto.items.push(item_dto);
//             }
//         } else {
//             topic_data.standalone_items.push(item_dto);
//         }
//     }

//     // --- Step 5: Assemble the final response DTO ---
//     let root_data = data_by_topic
//         .remove(parent_topic_id)
//         .unwrap_or_else(|| TopicData {
//             standalone_items: Vec::new(),
//             passages: HashMap::new(),
//         });

//     let root_items_and_passages = ItemsAndPassages {
//         items: root_data.standalone_items,
//         passages: root_data.passages.into_values().collect(),
//     };

//     let subtopics_dto = subtopic_ids
//         .into_iter()
//         .filter_map(|sub_id| {
//             data_by_topic.remove(&sub_id).map(|data| {
//                 let items_and_passages = ItemsAndPassages {
//                     items: data.standalone_items,
//                     passages: data.passages.into_values().collect(),
//                 };
//                 SubtopicDto {
//                     id: sub_id.clone(),
//                     name: topic_map.get(&sub_id).cloned().unwrap_or_default(),
//                     // The DTO expects a Vec, so we wrap it
//                     items: vec![items_and_passages],
//                 }
//             })
//         })
//         .collect();

//     Ok(TopicItemsDto {
//         topic_id: parent_topic_id.to_string(),
//         task_id: task_id.to_string(),
//         items: root_items_and_passages,
//         subtopics: subtopics_dto,
//     })
// }

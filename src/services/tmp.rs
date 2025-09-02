#![allow(unused_imports, dead_code)]
use diesel::{prelude::*, sql_types::Text};
use serde::Serialize;
use std::sync::Arc;
use std::{borrow::BorrowMut, collections::HashMap};

use crate::models::passages::Passage;
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
    schema::{item_options, items, passages, topics},
};


pub fn fetch_topic_items_with_subtopics(
    parent_topic_id: &str,
    task_id: &str,
    pool: Arc<DbPool>,
) -> Result<TopicItemsDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    // --- Step 1: Find all relevant topic IDs (parent and direct children) ---
    let subtopic_ids: Vec<String> = topics::table
        .filter(topics::parent_topic_id.eq(parent_topic_id))
        .select(topics::id)
        .load(&mut conn)?;

    let mut relevant_topic_ids = subtopic_ids.clone(); // Clone for later use
    relevant_topic_ids.push(parent_topic_id.to_string());

    // --- Step 2: Fetch all relevant models from the database ---
    let relevant_items = items::table
        .filter(items::topic_id.eq_any(&relevant_topic_ids))
        .filter(items::task_id.eq(task_id))
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

    // --- Step 3: Group options by item ID for efficient mapping ---
    let mut options_by_item: HashMap<String, Vec<ItemOptions>> = HashMap::new();
    for opt in all_options {
        options_by_item
            .entry(opt.item_id.clone())
            .or_default()
            .push(opt);
    }

    // --- Step 4: Process and organize all fetched items into DTOs ---
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
                        stem: passage_model.stem.clone(),
                        items: Vec::new(),
                    });
                passage_dto.items.push(item_dto);
            }
        } else {
            topic_data.standalone_items.push(item_dto);
        }
    }

    // --- Step 5: Assemble the final response DTO ---
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

// use crate::schema::{topics};

// pub fn fetch_subject_items_with_subtopics(
//     topic_id: &str,
//     task_id_: &str,
//     pool: Arc<DbPool>
// ) -> Result<TopicItemsDto, ModuleError> {

//     let mut conn = pool
//         .get()
//         .map_err(|e| ModuleError::InternalError(e.to_string()))?;

//     // Step 1: Join items with options and topics
//     let rows = items::table
//         .left_join(item_options::table.on(item_options::item_id.eq(items::id)))
//         .left_join(topics::table.on(items::topic_id.eq(topics::id)))
//         .filter(items::topic_id.eq(topic_id))
//         .filter(items::task_id.eq(task_id_))
//         .select((
//             items::id,
//             items::title,
//             items::text,
//             items::question_type,
//             items::difficulty,
//             items::status,
//             topics::id.nullable(),
//             topics::name.nullable(),
//             topics::parent_topic_id.nullable(),
//             item_options::id.nullable(),
//             item_options::label.nullable(),
//             item_options::value.nullable(),
//             item_options::is_answer.nullable(),
//         ))
//         .load::<(
//             String, String, String, String, i16, String,
//             Option<String>, Option<String>, Option<String>,
//             Option<String>, Option<String>, Option<i64>, Option<bool>
//         )>(&mut conn)?;

//     // Step 2: Organize into DTOs
//     use std::collections::HashMap;

//     let mut items_map: HashMap<String, ItemDto> = HashMap::new();
//     let mut subtopics_map: HashMap<String, SubtopicDto> = HashMap::new();
//     let mut root_items: Vec<ItemDto> = Vec::new();

//     for row in rows {
//         let (
//             item_id, title, text, question_type, difficulty, status,
//             topic_id, topic_name, parent_topic_id,
//             opt_id, opt_label, opt_value, opt_is_answer
//         ) = row;

//         let item_entry = items_map.entry(item_id.clone()).or_insert(ItemDto {
//             id: item_id.clone(),
//             title: title.clone(),
//             text: text.clone(),
//             question_type: question_type.clone(),
//             difficulty,
//             status: status.clone(),
//             options: Vec::new(),
//         });

//         if let (Some(oid), Some(lbl), Some(val), Some(ans)) = (opt_id, opt_label, opt_value, opt_is_answer) {
//             item_entry.options.push(OptionDto {
//                 id: oid,
//                 label: lbl,
//                 value: val,
//                 is_answer: ans,
//             });
//         }

//         match (topic_id, topic_name, parent_topic_id) {
//             // Root level (no subtopic)
//             (None, _, _) => {
//                 if !root_items.iter().any(|i| i.id == item_entry.id) {
//                     root_items.push(item_entry.clone());
//                 }
//             }
//             // Belongs to a subtopic
//             (Some(tid), Some(tname), Some(_parent)) => {
//                 let subtopic = subtopics_map.entry(tid.clone()).or_insert(SubtopicDto {
//                     id: tid.clone(),
//                     name: tname.clone(),
//                     items: Vec::new(),
//                 });

//                 if !subtopic.items.iter().any(|i| i.id == item_entry.id) {
//                     subtopic.items.push(item_entry.clone());
//                 }
//             }
//             _ => {} // ignore malformed cases
//         }
//     }

//     Ok(TopicItemsDto {
//         topic_id: topic_id.to_string(),
//         task_id: task_id_.to_string(),
//         items: root_items,
//         subtopics: subtopics_map.into_values().collect(),
//     })
// }




pub fn fetch_topic_items_with_subtopics(
    parent_topic_id: &str,
    task_id: &str,
    pool: Arc<DbPool>,
) -> Result<TopicItemsDto, ModuleError> {
    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    // Find all relevant topic IDs (the parent and its direct children) ---
    let subtopic_ids: Vec<String> = topics::table
        .filter(topics::parent_topic_id.eq(parent_topic_id))
        .select(topics::id)
        .load(&mut conn)?;

    let mut relevant_topic_ids = subtopic_ids;
    relevant_topic_ids.push(parent_topic_id.to_string());

    // Fetch all relevant items and topics in separate, efficient queries ---
    let relevant_items = items::table
        .filter(items::topic_id.eq_any(&relevant_topic_ids))
        .filter(items::task_id.eq(task_id))
        .select(Items::as_select()) // Assuming a model `Item` derived with `Selectable`
        .load::<Items>(&mut conn)?;

    // Create a map of topic_id -> topic_name for easy lookup later
    let topic_map: HashMap<String, String> = topics::table
        .filter(topics::id.eq_any(&relevant_topic_ids))
        .select((topics::id, topics::name))
        .load::<(String, String)>(&mut conn)?
        .into_iter()
        .collect();

    // Fetch all options for these items using Diesel's associations ---
    let all_options = crate::schema::item_options::table
        .filter(
            crate::schema::item_options::item_id
                .eq_any(&relevant_items.iter().map(|i| &i.id).collect::<Vec<_>>()),
        )
        .select(ItemOptions::as_select()) // Assuming a model `ItemOption`
        .load::<ItemOptions>(&mut conn)?;

    // Group options by their parent item_id for efficient mapping (manual grouping)
    let mut options_by_item: HashMap<String, Vec<ItemOptions>> = HashMap::new();
    for opt in all_options {
        options_by_item
            .entry(opt.item_id.clone())
            .or_default()
            .push(opt);
    }

    // Combine the fetched data into DTOs ---
    let items_with_options: Vec<(Items, Vec<ItemOptions>)> = relevant_items
        .into_iter()
        .map(|it| {
            let opts = options_by_item.remove(&it.id).unwrap_or_default();
            (it, opts)
        })
        .collect();

    let mut root_items = Vec::new();
    let mut subtopics_map: HashMap<String, SubtopicDto> = HashMap::new();

    for (item, options) in items_with_options {
        // Map the database model to the DTO
        let item_dto = ItemDto {
            id: item.id,
            title: item.title,
            text: item.text,
            question_type: format!("{:?}", item.question_type),
            difficulty: item.difficulty,
            status: format!("{:?}", item.status),
            options: options
                .into_iter()
                .map(|opt| OptionDto {
                    id: opt.id,
                    label: opt.label,
                    value: opt.value,
                    is_answer: opt.is_answer,
                })
                .collect(),
        };

        // Place the item DTO in the correct category (root or subtopic)
        if item.topic_id == parent_topic_id {
            root_items.push(item_dto);
        } else {
            let topic_name = topic_map.get(&item.topic_id).cloned().unwrap_or_default();

            subtopics_map
                .entry(item.topic_id.clone())
                .or_insert_with(|| SubtopicDto {
                    id: item.topic_id,
                    name: topic_name,
                    items: Vec::new(),
                })
                .items
                .push(item_dto);
        }
    }

    Ok(TopicItemsDto {
        topic_id: parent_topic_id.to_string(),
        task_id: task_id.to_string(),
        items: root_items,
        subtopics: subtopics_map.into_values().collect(),
    })
}


pub mod types{

#[derive(Debug, Serialize, Clone)]
pub struct OptionDto {
    pub id: String,
    pub label: String,
    pub value: i64,
    pub is_answer: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ItemDto {
    pub id: String,
    pub title: String,
    pub text: String,
    pub question_type: String,
    pub difficulty: i16,
    pub status: String,
    pub options: Vec<OptionDto>,
}

#[derive(Debug, Serialize)]
pub struct SubtopicDto {
    pub id: String,
    pub name: String,
    pub items: Vec<ItemDto>,
}

#[derive(Debug, Serialize)]
pub struct TopicItemsDto {
    pub topic_id: String,
    pub task_id: String,
    pub items: Vec<ItemDto>,
    pub subtopics: Vec<SubtopicDto>,
}
}
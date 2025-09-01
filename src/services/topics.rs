use std::{collections::HashMap, sync::Arc};

use crate::{
    DbPool,
    error::ModuleError,
    helpers::{self, dto::topic::*},
};

use diesel::prelude::*;

pub fn fetch_subject_topics(
    subject_id: &str,
    pool: Arc<DbPool>,
) -> Result<Vec<TopicNode>, ModuleError> {
    use crate::schema::{tasks::dsl as tk, topics::dsl as t};

    let mut conn = pool
        .get()
        .map_err(|e| ModuleError::InternalError(e.to_string()))?;

    let active_task = helpers::get_current_user_task(&mut conn)?;
    if let Some(task_id) = active_task {
        let flat_topics = t::topics
            .left_join(tk::tasks.on(tk::topic_id.eq(t::id).and(tk::subject_id.eq(t::subject_id))))
            .filter(t::subject_id.eq(subject_id))
            .filter(tk::task_id.eq(task_id))
            .select((
                t::id,
                t::name,
                t::parent_topic_id,
                tk::task_id.nullable(),
                tk::num_of_questions.nullable(),
            ))
            .load::<FlatTopic>(&mut conn)?;
        return Ok(build_hierarchy(flat_topics));
    }

    Ok(vec![])
}

pub fn fetch_subtopics_under_topic(
    subject_id: &str,
    topic_id: &str,
    pool: Arc<DbPool>,
) -> Result<Vec<TopicNode>, ModuleError> {
    let tree = fetch_subject_topics(subject_id, pool)?;
    if let Some(subs) = tree.iter().find_map(|t| t.find_subtopics(topic_id)) {
        return Ok(subs);
    }
    Ok(vec![])
}

pub fn build_hierarchy(flat_topics: Vec<FlatTopic>) -> Vec<TopicNode> {
    let mut children_by_parent: HashMap<Option<String>, Vec<FlatTopic>> = HashMap::new();
    for topic in flat_topics {
        children_by_parent
            .entry(topic.parent_topic_id.clone())
            .or_default()
            .push(topic);
    }

    fn build_nodes_for_parent(
        parent_id: Option<&String>,
        children_map: &mut HashMap<Option<String>, Vec<FlatTopic>>,
    ) -> Vec<TopicNode> {
        match children_map.remove(&parent_id.cloned()) {
            Some(children) => children
                .into_iter()
                .map(|child_topic| {
                    let mut subtopics =
                        build_nodes_for_parent(Some(&child_topic.topic_id), children_map);

                    // roll-up total: own + children
                    let mut total = child_topic.num_of_questions.unwrap_or(0) as i64;
                    for sub in &mut subtopics {
                        total += sub.expected_total_count;
                    }

                    TopicNode {
                        id: child_topic.topic_id,
                        name: child_topic.topic_name,
                        num_of_questions: child_topic.num_of_questions,
                        expected_total_count: total,
                        task_id: child_topic.task_id,
                        subtopics,
                    }
                })
                .collect(),
            None => Vec::new(),
        }
    }

    build_nodes_for_parent(None, &mut children_by_parent)
}

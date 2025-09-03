pub const GET_SUBJECT_ITEM_READY_DETAILS: &str = r#"
SELECT 
    t.topic_id,
    t.topic_name,
    t.num_of_questions AS expected,
    COUNT(i.id) FILTER (WHERE i.status = '"Ready"') AS ready
FROM tasks t
LEFT JOIN items i 
    ON i.task_id = t.task_id 
   AND i.topic_id = t.topic_id
   AND i.subject_id = t.subject_id
WHERE t.subject_id = $1
  AND t.task_id = $2
GROUP BY t.topic_id, t.topic_name, t.num_of_questions;
"#;

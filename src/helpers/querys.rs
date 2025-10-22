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

pub const GET_SUBJECT_ITEM_TOTAL_STATS: &str = r#"
SELECT
    t.id AS topic_id,
    t.name AS topic_name,
    COALESCE(ts.num_of_questions, 0) AS expected_items,
    COUNT(i.id) FILTER (WHERE i.status = '"Draft"') AS items_in_draft,
    COUNT(i.id) FILTER (WHERE i.status = '"Ready"') AS ready_items,
    COUNT(i.id) FILTER (WHERE i.status = '"Submitted"') AS submitted_items
FROM topics t
LEFT JOIN tasks ts
    ON ts.topic_id = t.id
   AND ts.subject_id = t.subject_id
LEFT JOIN items i
    ON i.topic_id = t.id
   AND i.subject_id = t.subject_id

WHERE t.subject_id = $1

GROUP BY t.id, t.name
ORDER BY t.name;
"#;

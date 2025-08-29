// @generated automatically by Diesel CLI.

diesel::table! {
    activity_logs (id) {
        id -> Text,
        task_id -> Nullable<Text>,
        activity_type -> Text,
        target_id -> Nullable<Text>,
        user_id -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    item_options (id) {
        id -> Text,
        item_id -> Text,
        label -> Text,
        value -> BigInt,
        is_answer -> Bool,
    }
}

diesel::table! {
    items (id) {
        id -> Text,
        subject_id -> Text,
        topic_id -> Text,
        question_type -> Text,
        stem -> Text,
        rubric -> Text,
        difficulty -> SmallInt,
        status -> Text,
        created_by -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        passage_id -> Nullable<Text>,
        used -> Text,
        reviewer_id_one -> Nullable<Text>,
        reviewer_id_two -> Nullable<Text>,
        mg_id -> Nullable<Text>,
        mg_passage_id -> Nullable<Text>,
        edit_level -> Text,
        taxonomy -> Text,
        task_id -> Text,
        count -> BigInt,
    }
}

diesel::table! {
    passages (id) {
        id -> Text,
        stem -> Text,
        topic_id -> Text,
        created_by -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        subject_id -> Text,
        mg_passage_id -> Nullable<Text>,
        mg_id -> Nullable<Text>,
    }
}

diesel::table! {
    topics (id, subject_id) {
        id -> Text,
        subject_id -> Text,
        parent_topic_id -> Nullable<Text>,
        name -> Text,
        created_by -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        rubric -> Text,
        archived -> Bool,
    }
}

diesel::table! {
    user (id) {
        id -> Text,
        email -> Text,
        password_hash -> Text,
        first_name -> Text,
        last_name -> Text,
        is_active -> Bool,
        session_id -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        username -> Text,
        department -> Text,
        institution -> Text,
        title -> Text,
        bearer_token -> Text,
    }
}

diesel::table! {
    tasks(task_id,topic_id){
    task_id -> Text,
    subject_id -> Text,
    subject_name -> Text,
    topic_id -> Text,
    topic_name -> Text,
    num_of_questions -> Integer,
    subject_code -> VarChar,
    start_date -> Timestamp,
    due_date -> Timestamp
    }
}

diesel::allow_tables_to_appear_in_same_query!(activity_logs, item_options, items, passages, user,);

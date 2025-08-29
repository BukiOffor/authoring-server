-- Your SQL goes here
CREATE TABLE `user`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`email` VARCHAR NOT NULL,
	`password_hash` TEXT NOT NULL,
	`first_name` VARCHAR NOT NULL,
	`last_name` VARCHAR NOT NULL,
	`is_active` BOOL NOT NULL,
	`session_id` TEXT,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`username` VARCHAR NOT NULL,
	`department` VARCHAR NOT NULL,
	`institution` VARCHAR NOT NULL,
	`title` VARCHAR NOT NULL,
	`bearer_token` TEXT NOT NULL
);

CREATE TABLE `activity_logs`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`task_id` TEXT,
	`activity_type` TEXT NOT NULL,
	`target_id` TEXT,
	`user_id` TEXT NOT NULL,
	`created_at` TIMESTAMP NOT NULL
);

CREATE TABLE `items`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`subject_id` TEXT NOT NULL,
	`topic_id` TEXT NOT NULL,
	`question_type` VARCHAR NOT NULL,
	`stem` TEXT NOT NULL,
	`rubric` TEXT NOT NULL,
	`difficulty` INT2 NOT NULL,
	`status` VARCHAR NOT NULL,
	`created_by` TEXT NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`passage_id` TEXT,
	`used` TEXT NOT NULL,
	`reviewer_id_one` TEXT,
	`reviewer_id_two` TEXT,
	`mg_id` TEXT,
	`mg_passage_id` TEXT,
	`edit_level` TEXT NOT NULL,
	`taxonomy` TEXT NOT NULL,
	`task_id` TEXT NOT NULL,
	`count` INT8 NOT NULL
);

CREATE TABLE `passages`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`stem` TEXT NOT NULL,
	`topic_id` TEXT NOT NULL,
	`created_by` TEXT NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`subject_id` TEXT NOT NULL,
	`mg_passage_id` TEXT,
	`mg_id` TEXT
);

CREATE TABLE `item_options`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`item_id` TEXT NOT NULL,
	`label` TEXT NOT NULL,
	`value` INT8 NOT NULL,
	`is_answer` BOOL NOT NULL
);


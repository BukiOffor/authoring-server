-- Your SQL goes here







CREATE TABLE `tos`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`subject_id` TEXT NOT NULL,
	`placement` INT4 NOT NULL,
	`num_of_questions` INT4 NOT NULL,
	`start_range` INT4 NOT NULL,
	`end_range` INT4 NOT NULL,
	`item_type` VARCHAR NOT NULL,
	`number_of_passages` INT4 NOT NULL,
	`total_items_in_passage` INT4 NOT NULL,
	`topic_id` TEXT NOT NULL,
	`sub_topic_id` TEXT NOT NULL
);


-- Your SQL goes here





CREATE TABLE `tasks`(
	`task_id` TEXT NOT NULL,
	`subject_id` TEXT NOT NULL,
	`subject_name` TEXT NOT NULL,
	`topic_id` TEXT NOT NULL,
	`topic_name` TEXT NOT NULL,
	`num_of_questions` INTEGER NOT NULL,
	`subject_code` VARCHAR NOT NULL,
	`start_date` TIMESTAMP NOT NULL,
	`due_date` TIMESTAMP NOT NULL,
	PRIMARY KEY(`task_id`, `topic_id`)
);

CREATE TABLE `topics`(
	`id` TEXT NOT NULL,
	`subject_id` TEXT NOT NULL,
	`parent_topic_id` TEXT,
	`name` TEXT NOT NULL,
	`created_by` TEXT NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`rubric` TEXT NOT NULL,
	`archived` BOOL NOT NULL,
	PRIMARY KEY(`id`, `subject_id`)
);


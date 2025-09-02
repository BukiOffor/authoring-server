-- This file should undo anything in `up.sql`


ALTER TABLE `items` DROP COLUMN `text`;
ALTER TABLE `items` DROP COLUMN `title`;
ALTER TABLE `items` ADD COLUMN `stem` TEXT NOT NULL;
ALTER TABLE `items` ADD COLUMN `rubric` TEXT NOT NULL;
ALTER TABLE `items` ADD COLUMN `created_by` TEXT NOT NULL;
ALTER TABLE `items` ADD COLUMN `used` TEXT NOT NULL;
ALTER TABLE `items` ADD COLUMN `reviewer_id_one` TEXT;
ALTER TABLE `items` ADD COLUMN `reviewer_id_two` TEXT;
ALTER TABLE `items` ADD COLUMN `mg_id` TEXT;
ALTER TABLE `items` ADD COLUMN `mg_passage_id` TEXT;
ALTER TABLE `items` ADD COLUMN `edit_level` TEXT NOT NULL;
ALTER TABLE `items` ADD COLUMN `count` BIGINT NOT NULL;

ALTER TABLE `passages` ADD COLUMN `mg_passage_id` TEXT;
ALTER TABLE `passages` ADD COLUMN `mg_id` TEXT;





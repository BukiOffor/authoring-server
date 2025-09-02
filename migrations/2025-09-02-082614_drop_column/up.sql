-- Your SQL goes here


ALTER TABLE `items` DROP COLUMN `stem`;
ALTER TABLE `items` DROP COLUMN `rubric`;
ALTER TABLE `items` DROP COLUMN `created_by`;
ALTER TABLE `items` DROP COLUMN `used`;
ALTER TABLE `items` DROP COLUMN `reviewer_id_one`;
ALTER TABLE `items` DROP COLUMN `reviewer_id_two`;
ALTER TABLE `items` DROP COLUMN `mg_id`;
ALTER TABLE `items` DROP COLUMN `mg_passage_id`;
ALTER TABLE `items` DROP COLUMN `edit_level`;
ALTER TABLE `items` DROP COLUMN `count`;
ALTER TABLE `items` ADD COLUMN `text` TEXT NOT NULL;
ALTER TABLE `items` ADD COLUMN `title` TEXT NOT NULL;

ALTER TABLE `passages` DROP COLUMN `mg_passage_id`;
ALTER TABLE `passages` DROP COLUMN `mg_id`;





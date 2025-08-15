-- 修复offset列名与PostgreSQL关键字冲突
-- 将tags表中的"offset"列重命名为"tag_offset"

ALTER TABLE tags RENAME COLUMN "offset" TO tag_offset;
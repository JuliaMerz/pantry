-- This file should undo anything in `up.sql`
ALTER TABLE llm
RENAME COLUMN local TO create_thread;

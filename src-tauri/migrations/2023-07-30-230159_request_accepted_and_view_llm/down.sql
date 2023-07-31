-- This file should undo anything in `up.sql`

ALTER TABLE requests
	DROP accepted;
ALTER TABLE user
	DROP perm_view_llms;

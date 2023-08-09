-- Your SQL goes here
ALTER TABLE user
	ADD perm_bare_model BOOLEAN DEFAULT FALSE NOT NULL;

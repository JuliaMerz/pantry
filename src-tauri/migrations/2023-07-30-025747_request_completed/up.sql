-- Your SQL goes here

ALTER TABLE requests
	ADD complete BOOLEAN DEFAULT FALSE NOT NULL;

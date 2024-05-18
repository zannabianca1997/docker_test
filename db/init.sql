-- Connect to the 'messages' database
\c messages

-- Create the 'messages' table
CREATE TABLE messages (
    time TIMESTAMP NOT NULL,
    "user" VARCHAR(32) NOT NULL,
    content TEXT NOT NULL
);
# Schema notes for memos-rs

## Goal
Define a minimal database schema for the Rust MVP, not full Memos parity.

## P0 tables

### users
Fields:
- id
- username
- display_name
- email_nullable
- password_hash
- role
- created_at
- updated_at

Notes:
- first created user becomes admin
- keep role simple at first: admin / user

### memos
Fields:
- id
- creator_id
- content
- visibility
- pinned
- archived
- created_at
- updated_at

Notes:
- content stored as plain markdown text
- visibility can start simple: private / public / unlisted if needed
- full upstream parity is not required in P0

### sessions_or_tokens
Fields:
- id
- user_id
- token_id_or_jti
- expires_at
- created_at

Notes:
- only needed if using revocable sessions
- if using stateless JWT only, keep this optional

## P1 candidate tables
- tags
- memo_tags
- resources
- attachments

## Migration rules
- use sqlx migrations
- each migration must be reversible where practical
- avoid speculative columns until required

## Compatibility strategy
- optimize for clean Rust schema first
- only preserve upstream field names when it materially helps frontend/API compatibility
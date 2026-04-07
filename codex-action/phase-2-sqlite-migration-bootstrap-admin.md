Task: add persistent SQLite support and bootstrap initialization.

Implement the following:

1. Database layer:
- add sqlx with SQLite
- add migrations directory and initial migration
- create a database connection pool
- wire database state into the application

2. Initial schema:
- users table
- memos table
- sessions or tokens table if needed
- created_at / updated_at timestamps
- reasonable indexes

3. Bootstrap behavior:
- if no users exist, allow creation of the first admin user
- add a bootstrap endpoint or CLI flow for first-run setup
- persist password hashes using argon2

4. Config:
- add database path configuration
- keep SQLite as default
- prepare config structure so PostgreSQL/MySQL can be added later

5. Validation:
- add at least basic integration or unit tests for DB init and bootstrap flow

Output requirements:
- explain schema decisions
- show modified file tree
- implement the files
- show migration commands
- show exact local test commands

Keep the code aligned with the current project layout.
Do not implement advanced Memos features yet.
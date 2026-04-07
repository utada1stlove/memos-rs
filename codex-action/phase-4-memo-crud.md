Task: implement the core memo CRUD APIs.

Requirements:
- create memo
- get memo by id
- update memo
- delete memo
- list memos
- support basic filtering by creator and simple ordering by created time
- support plain markdown text storage
- authenticated routes for create/update/delete
- sensible JSON request and response structures

Data model:
- memo id
- creator id
- content
- visibility if practical
- pinned / archived can be deferred unless easy to include
- created_at / updated_at

Engineering requirements:
- clear module separation under src/memo
- database queries via sqlx
- tests where practical
- avoid premature complex search functionality

Documentation:
- show curl examples for all CRUD endpoints
- explain any intentionally deferred features
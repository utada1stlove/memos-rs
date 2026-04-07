Task: implement authentication for memos-rs.

Requirements:
- login endpoint
- password verification using argon2
- token generation using JWT or equivalent standard approach
- auth middleware for protected endpoints
- logout handling if your token/session model supports it
- clear error responses for invalid credentials and unauthorized access

Design constraints:
- keep implementation simple and production-sane
- do not over-engineer RBAC yet
- prepare for multi-user support later
- keep the first admin bootstrap flow working

Documentation:
- document the auth flow
- show how to login with curl
- show how to call a protected endpoint

Testing:
- add tests for successful login
- add tests for invalid login
- add tests for protected route access without token
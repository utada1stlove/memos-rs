Task: add static frontend hosting support.

Goal:
Allow memos-rs to serve a built frontend directory so the application can be deployed as a single service.

Requirements:
- configurable static assets directory
- serve index.html and frontend assets
- keep API routes under /api or another clear prefix
- preserve health endpoint
- avoid breaking backend route structure

Do not rewrite the frontend.
Only make the backend capable of hosting prebuilt frontend files.

Documentation:
- describe expected frontend build output location
- show how to run backend with static assets enabled
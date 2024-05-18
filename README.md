# TestChat

This is an **exercise** to learn Docker. 
It's a basic chatroom, overengineered to test more of Docker capacities.
As security is not an objective, expect the project to be ample vulnerable to exploits.

# Running
Simply launch with `docker compose`.

For a local test deploy:
```bash
TITLE='...' docker compose up
```

For a public deploy:
```bash
TITLE='...' PUBLIC_IP='...' docker compose --env-file .env.public up
```
Where `PUBLIC_IP` is assigned to the IP where your clients can reach the API. You can also set directly `API_URL` if you need to customize the port or protocol.
# a test script that generates a ton of users for debugging use
# note that you'll need to comment out the ratelimiter in delta/src/main.rs
# and keep the number relatively low or requests will time out (the beefier the machine the more you can handle).
# this script assumes mailhog is running, and uses that to automate "emails".

# In the real world, antispam will catch this and nuke you to hell and back.
# But it works fine in a dev env!

# requires httpx
import asyncio
import os
import re
import uuid

import httpx

API_URL: str = os.getenv("API_URL") # type: ignore
MAILHOG_API: str = os.getenv("MAILHOG_API") # type: ignore
COUNT = int(os.getenv("COUNT")) # type: ignore # cbf to deal with type checking
INVITE: str = os.getenv("INVITE") # type: ignore

assert API_URL and MAILHOG_API and COUNT and INVITE

API_URL = API_URL.strip("/")
MAILHOG_API = MAILHOG_API.strip("/")

async def filter_hog(client: httpx.AsyncClient, email: str) -> str:
    """
    returns the token provided by the mail server.
    This script assumes the use of mailhog.
    """

    resp = await client.get(MAILHOG_API + "/api/v2/search", params={"kind": "to", "query": email}, follow_redirects=True, timeout=60)
    if resp.status_code != 200:
        raise Exception(resp.status_code, resp.content)
    
    data = resp.json()
    if not data["items"]:
        raise Exception("No message found")
    
    message_id = data["items"][0]["ID"]
    body = data["items"][0]["Content"]["Body"].replace("\r", "")
    token = re.search("/login/verify(=\n/|/\n=|/=\n)(?P<token>[^\n]+)", body, re.MULTILINE)
    if not token:
        raise Exception("No token found")
    
    ret = token.group("token")

    await client.delete(MAILHOG_API + f"/api/v1/messages/{message_id}", timeout=60)
    return ret


async def task() -> None:
    _id = str(uuid.uuid4())[:4]
    email = f"{_id}@example.com"

    async with httpx.AsyncClient() as client:
        resp = await client.post(API_URL + "/auth/account/create", json={"email": email, "password": _id*3, "invite": INVITE}, timeout=60)
        if resp.status_code != 204:
            raise Exception(resp.status_code, resp.content)

        token = await filter_hog(client, email)

        resp = await client.post(API_URL + f"/auth/account/verify/{token}", timeout=60)
        if resp.status_code != 200:
            raise Exception("verify", resp.status_code, resp.content)
        
        ticket = resp.json()["ticket"]
        userid = ticket["_id"]
        
        resp = await client.post(API_URL + "/auth/session/login", json={"email": email, "password": _id*3, "friendly_name": "Not A Client"}, timeout=60)
        if resp.status_code != 200:
            raise Exception("session", resp.status_code, resp.content)
        
        session = resp.json()
        token = session["token"]
        
        resp = await client.post(API_URL + "/onboard/complete", json={"username": _id}, headers={"x-session-token": token}, timeout=60) # complete onboarding to allow creating a session
        if resp.status_code != 200:
            raise Exception("onboard", resp.status_code, resp.content)
        
        resp = await client.post(API_URL + f"/invites/{INVITE}", headers={"x-session-token": token}, timeout=60)
        if resp.status_code != 200:
            raise Exception("invite", resp.status_code, resp.content)
        
        print(f"Created account and session for {email} with ID: {userid}")
        return userid

async def main():
    tasks = [asyncio.create_task(task()) for _ in range(COUNT)]
    print(await asyncio.gather(*tasks))

asyncio.run(main())
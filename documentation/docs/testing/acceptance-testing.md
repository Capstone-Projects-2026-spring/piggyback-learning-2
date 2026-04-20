---
sidebar_position: 3
---

# Acceptance Test

### AT-01: Homepage Loads Successfully

**Actions Taken:**
curl -i http://127.0.0.1:3000/
OR
visit http://localhost:3000/ in your favorite browser

**Observed Result:**

-   HTTP/1.1 200 OK

-   HTML page rendered successfully

-   Page title displayed: "Educational Video Platform"

**Status:** PASS

* * * * *

### AT-02: Create an Account and Log In

**Actions Taken:**
- visit http://localhost:3000/
- click "Sign up"
- Create a new parent account
- Sign into the account

**Observed Result:**

-   Account is created and able to be logged in to

**Status:** PASS

* * * * *


### AT-03: Mood Detection

**Actions Taken:**

- Begin watching a video with questions generated
- Look away from the screen or leave the camera view. Video should pause
- Do not answer a question or answer in a very low voice. Should be flagged as bored (will be able to check in video results).
- Check the video results.

**Observed Result:**

- Video pauses when looking away from screen.
- Answer is flagged as "Bored" in the results screen

**Status:** PASS


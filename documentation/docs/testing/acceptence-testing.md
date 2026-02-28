---
sidebar_position: 3
---
# Acceptance test
Automated Acceptance Test Results**Environment:** Django 6.0.2 running on Daphne ASGI server**Base URL:** [http://127.0.0.1:8000/](http://127.0.0.1:8000/)


AT-AUTO-01: Homepage Loads Successfully


**Objective:**Verify that the main web application loads without server errors.

**Command Executed:**curl -i http://127.0.0.1:8000/

**Observed Result:**

*   HTTP/1.1 200 OK returned
    
*   HTML page content rendered
    
*   Page title displayed: “Educational Video Platform”
    

**Status:** PASS

AT-AUTO-02: API Documentation (Swagger UI) Accessible


**Objective:**Confirm that API documentation is properly configured and accessible.

**Command Executed:**curl -i http://127.0.0.1:8000/api/docs/

**Observed Result:**

*   HTTP/1.1 200 OK returned
    
*   Swagger UI HTML content loaded successfully
    

**Status:** PASS


AT-AUTO-03: Protected Admin Route Redirects When Not Authenticated


**Objective:**Ensure restricted routes enforce authentication.

**Command Executed:**curl -i http://127.0.0.1:8000/django-admin/

**Observed Result:**

*   HTTP/1.1 302 Found returned
    
*   Redirected to /django-admin/login/?next=/django-admin/
    

**Status:** PASS

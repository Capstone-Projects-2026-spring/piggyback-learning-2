from fastapi.templating import Jinja2Templates
from app.settings import TEMPLATES_DIR

# Shared Jinja template loader used by route modules.
templates = Jinja2Templates (directory = str(TEMPLATES_DIR))
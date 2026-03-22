#!/usr/bin/env bash

#exit on error
set -o errexit

pip install -r requirements.txt

#convert static asset files (needed for render deploy)
python manage.py collectstatic --no-input

python manage.py migrate
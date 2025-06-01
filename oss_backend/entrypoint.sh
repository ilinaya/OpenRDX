#!/bin/sh
set -e

echo "⏳ Waiting for PostgreSQL at $DB_HOST:$DB_PORT..."
while ! nc -z "$DB_HOST" "$DB_PORT"; do
  sleep 1
done
echo "✅ PostgreSQL is available"

echo "🚀 Running Django migrations..."
python manage.py migrate

echo "🔫 Starting Gunicorn..."
exec gunicorn core.wsgi:application --bind 0.0.0.0:8000
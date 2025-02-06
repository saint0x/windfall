#!/bin/bash
set -e

echo "Setting up database..."

# Create data directory if it doesn't exist
mkdir -p data

# Remove existing database if it exists
if [ -f "data/windfall.db" ]; then
    echo "Removing existing database..."
    rm data/windfall.db
fi

# Initialize new database
echo "Creating new database..."
sqlite3 data/windfall.db ".databases"

# Run all migrations in order
echo "Running migrations..."
for migration in migrations/*.sql; do
    echo "Applying migration: $migration"
    sqlite3 data/windfall.db < "$migration"
done

echo "Database setup complete!" 
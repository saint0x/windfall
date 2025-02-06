#!/bin/bash
set -e

echo "Initializing database..."

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

# Apply the setup script
echo "Applying database setup..."
sqlite3 data/windfall.db < migrations/20240322000000_setup_db.sql

echo "Database initialization complete!" 
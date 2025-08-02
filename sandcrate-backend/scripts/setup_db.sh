#!/bin/bash

# Database setup script for Sandcrate
set -e

echo "🚀 Setting up PostgreSQL database for Sandcrate..."

# Check if PostgreSQL is installed
if ! command -v psql &> /dev/null; then
    echo "❌ PostgreSQL is not installed. Please install PostgreSQL first."
    echo "   Ubuntu/Debian: sudo apt-get install postgresql postgresql-contrib"
    echo "   macOS: brew install postgresql"
    echo "   Windows: Download from https://www.postgresql.org/download/windows/"
    exit 1
fi

# Check if PostgreSQL service is running
if ! pg_isready -q; then
    echo "❌ PostgreSQL service is not running. Please start PostgreSQL first."
    echo "   Ubuntu/Debian: sudo systemctl start postgresql"
    echo "   macOS: brew services start postgresql"
    exit 1
fi

# Create database and user
echo "📦 Creating database and user..."

# Create user (if it doesn't exist)
psql -U postgres -c "CREATE USER sandcrate WITH PASSWORD 'sandcrate';" 2>/dev/null || echo "User sandcrate already exists"

# Create database (if it doesn't exist)
psql -U postgres -c "CREATE DATABASE sandcrate OWNER sandcrate;" 2>/dev/null || echo "Database sandcrate already exists"

# Grant privileges
psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE sandcrate TO sandcrate;"

echo "✅ Database setup completed!"
echo ""
echo "📋 Next steps:"
echo "1. Copy env.example to .env and update the DATABASE_URL if needed"
echo "2. Run migrations: cargo sqlx migrate run"
echo "3. Start the backend: cargo run"
echo ""
echo "🔗 Database connection:"
echo "   Host: localhost"
echo "   Port: 5432"
echo "   Database: sandcrate"
echo "   User: sandcrate"
echo "   Password: sandcrate" 
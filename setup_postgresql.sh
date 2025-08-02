#!/bin/bash

set -e

echo "Setting up PostgreSQL for Sandcrate..."
echo "=========================================="

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_status "Checking PostgreSQL installation..."
if ! command -v psql &> /dev/null; then
    print_error "PostgreSQL is not installed."
    echo ""
    echo "Please install PostgreSQL first:"
    echo ""
    echo "Ubuntu/Debian:"
    echo "  sudo apt-get update"
    echo "  sudo apt-get install postgresql postgresql-contrib"
    echo "  sudo systemctl start postgresql"
    echo "  sudo systemctl enable postgresql"
    echo ""
    echo "macOS:"
    echo "  brew install postgresql"
    echo "  brew services start postgresql"
    echo ""
    echo "Windows:"
    echo "  Download from https://www.postgresql.org/download/windows/"
    exit 1
fi

print_success "PostgreSQL is installed"

print_status "Checking PostgreSQL service..."
if ! pg_isready -q; then
    print_error "PostgreSQL service is not running."
    echo ""
    echo "Please start PostgreSQL:"
    echo "  Ubuntu/Debian: sudo systemctl start postgresql"
    echo "  macOS: brew services start postgresql"
    exit 1
fi

print_success "PostgreSQL service is running"

PG_VERSION=$(psql --version | grep -oP '\d+\.\d+' | head -1)
print_status "PostgreSQL version: $PG_VERSION"

print_status "Setting up database and user..."

if psql -U postgres -c "SELECT 1 FROM pg_roles WHERE rolname='sandcrate'" | grep -q 1; then
    print_warning "User 'sandcrate' already exists"
else
    psql -U postgres -c "CREATE USER sandcrate WITH PASSWORD 'sandcrate';"
    print_success "Created user 'sandcrate'"
fi

if psql -U postgres -lqt | cut -d \| -f 1 | grep -qw sandcrate; then
    print_warning "Database 'sandcrate' already exists"
else
    psql -U postgres -c "CREATE DATABASE sandcrate OWNER sandcrate;"
    print_success "Created database 'sandcrate'"
fi

psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE sandcrate TO sandcrate;"
psql -U postgres -c "GRANT ALL ON SCHEMA public TO sandcrate;"

print_success "Database setup completed"

print_status "Checking SQLx CLI installation..."
if ! command -v sqlx &> /dev/null; then
    print_warning "SQLx CLI is not installed. Installing..."
    cargo install sqlx-cli --no-default-features --features postgres
    print_success "SQLx CLI installed"
else
    print_success "SQLx CLI is already installed"
fi

print_status "Setting up environment configuration..."
if [ ! -f "sandcrate-backend/.env" ]; then
    if [ -f "sandcrate-backend/env.example" ]; then
        cp sandcrate-backend/env.example sandcrate-backend/.env
        print_success "Created .env file from template"
    else
        print_warning "env.example not found, creating basic .env file"
        cat > sandcrate-backend/.env << EOF
DATABASE_URL=postgresql://sandcrate:sandcrate@localhost:5432/sandcrate
JWT_SECRET=your-super-secret
JWT_EXPIRATION_HOURS=24
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
PLUGINS_DIR=../assets/plugins
MAX_PLUGIN_SIZE_MB=50
LOG_LEVEL=info
EOF
        print_success "Created basic .env file"
    fi
else
    print_warning ".env file already exists"
fi

print_status "Running database migrations..."
cd sandcrate-backend

export DATABASE_URL="postgresql://sandcrate:sandcrate@localhost:5432/sandcrate"

if [ -d "migrations" ]; then
    if sqlx migrate run; then
        print_success "Database migrations completed"
    else
        print_error "Failed to run migrations"
        exit 1
    fi
else
    print_warning "No migrations directory found"
fi

print_status "Testing database connection..."
if psql -U sandcrate -d sandcrate -c "SELECT version();" > /dev/null 2>&1; then
    print_success "Database connection test passed"
else
    print_error "Database connection test failed"
    exit 1
fi

print_status "Initializing database with plugins..."
if cargo run --bin init_db 2>/dev/null || cargo run --example init_db 2>/dev/null; then
    print_success "Database initialization completed"
else
    print_warning "Database initialization script not found or failed"
fi

cd ..

echo ""
echo "=========================================="
print_success "PostgreSQL setup completed successfully!"
echo ""
echo "Configuration Summary:"
echo "  Database: sandcrate"
echo "  User: sandcrate"
echo "  Password: sandcrate"
echo "  Host: localhost"
echo "  Port: 5432"
echo ""
echo "Connection URL:"
echo "  postgresql://sandcrate:sandcrate@localhost:5432/sandcrate"
echo ""
echo "Environment file:"
echo "  sandcrate-backend/.env"
echo ""
echo "Next steps:"
echo "1. Start the backend: cd sandcrate-backend && cargo run"
echo "2. Start the frontend: cd sandcrate-react && npm run dev"
echo "3. Access the application at http://localhost:5173"
echo ""
echo "Documentation:"
echo "  See sandcrate-backend/DATABASE.md for detailed information"
echo "" 
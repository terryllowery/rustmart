# Lesson 16: Bash Scripting for Database Seeding and Automation

## Overview
Learn bash scripting to automate database seeding, environment setup, and common development tasks. You'll build scripts to generate realistic test data for RustMart, making it easier to test and demo your microservices.

By the end of this lesson, you'll have:
- Bash script to seed the database with random products
- Data generation using curl and jq
- Parameterized scripts for different data volumes
- Automation scripts for common development workflows
- Error handling and logging in bash

## Why Bash Scripting?

As a **DevOps/SRE professional**, bash is your Swiss Army knife:
- **Automation**: Repetitive tasks become one command
- **Portable**: Runs on any Unix-like system (macOS, Linux)
- **Glue code**: Connects different tools and services
- **CI/CD**: Most pipelines use bash scripts
- **Quick prototyping**: Faster than writing full programs

## Bash Basics Refresher

### Variables
```bash
#!/bin/bash

# Assignment (no spaces around =)
NAME="RustMart"
COUNT=100

# Using variables
echo "Hello, $NAME"
echo "Count: ${COUNT}"

# Command substitution
CURRENT_DIR=$(pwd)
DATE=$(date +%Y-%m-%d)
```

### Arrays
```bash
# Array declaration
NAMES=("Laptop" "Mouse" "Keyboard")

# Access elements
echo ${NAMES[0]}      # First element
echo ${NAMES[@]}      # All elements
echo ${#NAMES[@]}     # Array length

# Loop through array
for name in "${NAMES[@]}"; do
    echo "$name"
done
```

### Functions
```bash
generate_random_price() {
    local min=$1
    local max=$2
    echo $(( RANDOM % (max - min + 1) + min ))
}

# Call function
PRICE=$(generate_random_price 10 1000)
```

### Control Flow
```bash
# If statement
if [ $COUNT -gt 100 ]; then
    echo "Large dataset"
elif [ $COUNT -gt 10 ]; then
    echo "Medium dataset"
else
    echo "Small dataset"
fi

# For loop
for i in {1..10}; do
    echo "Iteration $i"
done

# While loop
COUNTER=0
while [ $COUNTER -lt 5 ]; do
    echo $COUNTER
    ((COUNTER++))
done
```

## Step 1: Basic Database Seeding Script

Create `scripts/seed-database.sh`:

```bash
#!/bin/bash

# Exit on error
set -e

# Configuration
API_BASE_URL="${API_BASE_URL:-http://localhost:8001}"
NUM_PRODUCTS="${NUM_PRODUCTS:-50}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if service is running
check_service() {
    log_info "Checking if product-service is running..."
    if ! curl -s -f "${API_BASE_URL}/health" > /dev/null; then
        log_error "Product service is not running at ${API_BASE_URL}"
        log_info "Start it with: cargo run -p product-service"
        exit 1
    fi
    log_info "Service is healthy!"
}

# Generate random product
create_product() {
    local index=$1
    
    # Product names
    local categories=("Laptop" "Desktop" "Tablet" "Phone" "Monitor" "Keyboard" "Mouse" "Webcam" "Headset" "Speaker")
    local brands=("Pro" "Elite" "Ultra" "Premium" "Standard" "Basic" "Gaming" "Office" "Studio" "Developer")
    
    # Random selection
    local category=${categories[$((RANDOM % ${#categories[@]}))]}
    local brand=${brands[$((RANDOM % ${#brands[@]}))]}
    local name="${brand} ${category}"
    
    # Random price between 50 and 2000
    local price=$(echo "scale=2; $(( RANDOM % 1950 + 50 )) + $(( RANDOM % 100 )) / 100" | bc)
    
    # Random inventory between 0 and 500
    local inventory=$((RANDOM % 501))
    
    # Create JSON payload
    local json=$(cat <<EOF
{
    "name": "${name}",
    "price": ${price},
    "inventory_count": ${inventory}
}
EOF
)
    
    # POST to API
    local response=$(curl -s -X POST "${API_BASE_URL}/products" \
        -H "Content-Type: application/json" \
        -d "$json")
    
    if [ $? -eq 0 ]; then
        log_info "Created product ${index}/${NUM_PRODUCTS}: ${name} - \$${price}"
    else
        log_error "Failed to create product: ${name}"
    fi
}

# Main execution
main() {
    log_info "Starting database seeding..."
    log_info "API URL: ${API_BASE_URL}"
    log_info "Number of products: ${NUM_PRODUCTS}"
    echo ""
    
    check_service
    echo ""
    
    log_info "Creating products..."
    for i in $(seq 1 $NUM_PRODUCTS); do
        create_product $i
    done
    
    echo ""
    log_info "✓ Database seeding complete!"
    log_info "Created ${NUM_PRODUCTS} products"
}

# Run main function
main
```

Make it executable:
```bash
chmod +x scripts/seed-database.sh
```

Run it:
```bash
# Seed with default 50 products
./scripts/seed-database.sh

# Seed with custom amount
NUM_PRODUCTS=100 ./scripts/seed-database.sh

# Use different API URL
API_BASE_URL=http://localhost:8000/products ./scripts/seed-database.sh
```

## Step 2: Advanced Data Generation with Realistic Data

Create `scripts/seed-realistic-data.sh`:

```bash
#!/bin/bash

set -e

API_BASE_URL="${API_BASE_URL:-http://localhost:8001}"
NUM_PRODUCTS="${NUM_PRODUCTS:-50}"

# Realistic product data
declare -A PRODUCTS=(
    # Laptops
    ["MacBook Pro 16"]="2499.99:15"
    ["Dell XPS 15"]="1899.99:25"
    ["ThinkPad X1 Carbon"]="1749.99:30"
    ["HP Spectre x360"]="1499.99:20"
    ["ASUS ROG Zephyrus"]="2299.99:10"
    
    # Monitors
    ["Dell UltraSharp 27"]="599.99:40"
    ["LG 34 Ultrawide"]="799.99:25"
    ["Samsung Odyssey G9"]="1299.99:15"
    ["BenQ SW270C"]="699.99:30"
    
    # Keyboards
    ["Keychron K2"]="89.99:100"
    ["Logitech MX Keys"]="119.99:80"
    ["Das Keyboard 4"]="169.99:50"
    ["Ducky One 3"]="149.99:60"
    
    # Mice
    ["Logitech MX Master 3S"]="99.99:120"
    ["Razer DeathAdder V3"]="69.99:90"
    ["SteelSeries Rival 3"]="29.99:150"
    
    # Audio
    ["Sony WH-1000XM5"]="399.99:50"
    ["Bose QuietComfort 45"]="329.99:60"
    ["Apple AirPods Pro"]="249.99:100"
    ["HyperX Cloud II"]="99.99:80"
)

GREEN='\033[0;32m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

create_realistic_product() {
    local name=$1
    local price=$2
    local inventory=$3
    
    local json=$(cat <<EOF
{
    "name": "${name}",
    "price": ${price},
    "inventory_count": ${inventory}
}
EOF
)
    
    curl -s -X POST "${API_BASE_URL}/products" \
        -H "Content-Type: application/json" \
        -d "$json" > /dev/null
    
    log_info "Created: ${name} - \$${price} (${inventory} in stock)"
}

main() {
    log_info "Seeding database with realistic products..."
    echo ""
    
    # Create all predefined products
    for product_name in "${!PRODUCTS[@]}"; do
        IFS=':' read -r price inventory <<< "${PRODUCTS[$product_name]}"
        create_realistic_product "$product_name" "$price" "$inventory"
    done
    
    echo ""
    log_info "✓ Created ${#PRODUCTS[@]} realistic products"
}

main
```

## Step 3: Bulk Operations with Parallel Execution

Speed up seeding with parallel requests:

Create `scripts/seed-parallel.sh`:

```bash
#!/bin/bash

set -e

API_BASE_URL="${API_BASE_URL:-http://localhost:8001}"
NUM_PRODUCTS="${NUM_PRODUCTS:-100}"
PARALLEL_JOBS="${PARALLEL_JOBS:-10}"

log_info() {
    echo "[INFO] $1"
}

create_product_batch() {
    local start=$1
    local end=$2
    
    for i in $(seq $start $end); do
        local name="Product-${i}"
        local price=$(echo "scale=2; $(( RANDOM % 1000 + 10 )) + $(( RANDOM % 100 )) / 100" | bc)
        local inventory=$((RANDOM % 200))
        
        curl -s -X POST "${API_BASE_URL}/products" \
            -H "Content-Type: application/json" \
            -d "{\"name\":\"${name}\",\"price\":${price},\"inventory_count\":${inventory}}" \
            > /dev/null 2>&1
    done
}

main() {
    log_info "Seeding ${NUM_PRODUCTS} products with ${PARALLEL_JOBS} parallel jobs..."
    
    local batch_size=$((NUM_PRODUCTS / PARALLEL_JOBS))
    
    for i in $(seq 0 $((PARALLEL_JOBS - 1))); do
        local start=$((i * batch_size + 1))
        local end=$(( (i + 1) * batch_size ))
        
        # Run in background
        create_product_batch $start $end &
    done
    
    # Wait for all background jobs
    wait
    
    log_info "✓ Completed seeding ${NUM_PRODUCTS} products"
}

main
```

Run with:
```bash
# Seed 1000 products with 20 parallel jobs
NUM_PRODUCTS=1000 PARALLEL_JOBS=20 ./scripts/seed-parallel.sh
```

## Step 4: Database Reset Script

Create `scripts/reset-database.sh`:

```bash
#!/bin/bash

set -e

DB_URL="${DATABASE_URL:-postgresql://rustmart_user:rustmart_pass@localhost/rustmart}"

RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

confirm() {
    log_warn "This will DELETE all data in the database!"
    read -p "Are you sure? (yes/no): " answer
    
    if [ "$answer" != "yes" ]; then
        echo "Aborted."
        exit 0
    fi
}

reset_database() {
    log_info "Truncating products table..."
    
    psql "$DB_URL" -c "TRUNCATE TABLE products CASCADE;" 2>/dev/null
    
    if [ $? -eq 0 ]; then
        log_info "✓ Database reset complete"
    else
        log_error "Failed to reset database"
        exit 1
    fi
}

main() {
    confirm
    reset_database
    
    log_info ""
    log_info "Database is now empty. You can seed it with:"
    log_info "  ./scripts/seed-database.sh"
}

main
```

## Step 5: Complete Dev Environment Setup

Create `scripts/dev-setup.sh`:

```bash
#!/bin/bash

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_step() {
    echo -e "\n${BLUE}==>${NC} $1\n"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    log_step "Checking dependencies..."
    
    local deps=("docker" "docker-compose" "psql" "cargo" "curl")
    
    for dep in "${deps[@]}"; do
        if command -v $dep &> /dev/null; then
            log_info "✓ $dep is installed"
        else
            log_error "✗ $dep is not installed"
            exit 1
        fi
    done
}

start_services() {
    log_step "Starting Docker services..."
    docker-compose up -d postgres kafka
    
    log_info "Waiting for services to be healthy..."
    sleep 5
}

run_migrations() {
    log_step "Running database migrations..."
    cd product-service
    cargo sqlx migrate run
    cd ..
}

seed_database() {
    log_step "Seeding database..."
    ./scripts/seed-realistic-data.sh
}

start_application() {
    log_step "Starting product-service..."
    log_info "Run in another terminal:"
    echo "  cargo run -p product-service"
}

main() {
    log_info "Setting up RustMart development environment..."
    
    check_dependencies
    start_services
    run_migrations
    seed_database
    start_application
    
    echo ""
    log_info "✓ Development environment ready!"
    log_info ""
    log_info "Quick commands:"
    log_info "  View products:  curl http://localhost:8001/products"
    log_info "  Health check:   curl http://localhost:8001/health"
    log_info "  Reset DB:       ./scripts/reset-database.sh"
}

main
```

## Step 6: Using jq for JSON Processing

Install jq (JSON processor):
```bash
brew install jq
```

Create `scripts/query-products.sh`:

```bash
#!/bin/bash

API_BASE_URL="${API_BASE_URL:-http://localhost:8001}"

# Get all products
get_all_products() {
    curl -s "${API_BASE_URL}/products"
}

# Pretty print products
list_products() {
    get_all_products | jq -r '.[] | "\(.id) - \(.name): $\(.price) (\(.inventory_count) in stock)"'
}

# Get products over $500
expensive_products() {
    get_all_products | jq '[.[] | select(.price > 500)]'
}

# Get low stock products (< 20)
low_stock() {
    get_all_products | jq '[.[] | select(.inventory_count < 20)]'
}

# Calculate total inventory value
total_value() {
    get_all_products | jq '[.[] | .price * .inventory_count] | add'
}

# Statistics
stats() {
    local data=$(get_all_products)
    
    echo "Product Statistics:"
    echo "==================="
    echo "Total products:    $(echo "$data" | jq 'length')"
    echo "Average price:     \$$(echo "$data" | jq '[.[].price] | add / length')"
    echo "Total inventory:   $(echo "$data" | jq '[.[].inventory_count] | add')"
    echo "Total value:       \$$(total_value)"
}

# Main menu
case "${1:-list}" in
    list)
        list_products
        ;;
    expensive)
        expensive_products
        ;;
    low-stock)
        low_stock
        ;;
    stats)
        stats
        ;;
    *)
        echo "Usage: $0 {list|expensive|low-stock|stats}"
        exit 1
        ;;
esac
```

Usage:
```bash
chmod +x scripts/query-products.sh

./scripts/query-products.sh list          # List all products
./scripts/query-products.sh expensive     # Products over $500
./scripts/query-products.sh low-stock     # Low inventory items
./scripts/query-products.sh stats         # Statistics
```

## Step 7: Backup and Restore Scripts

Create `scripts/backup-database.sh`:

```bash
#!/bin/bash

set -e

DB_URL="${DATABASE_URL:-postgresql://rustmart_user:rustmart_pass@localhost/rustmart}"
BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/rustmart_${TIMESTAMP}.sql"

GREEN='\033[0;32m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

mkdir -p "$BACKUP_DIR"

log_info "Creating database backup..."

pg_dump "$DB_URL" > "$BACKUP_FILE"

if [ $? -eq 0 ]; then
    log_info "✓ Backup created: $BACKUP_FILE"
    log_info "Size: $(du -h "$BACKUP_FILE" | cut -f1)"
    
    # Keep only last 10 backups
    ls -t ${BACKUP_DIR}/rustmart_*.sql | tail -n +11 | xargs -r rm
    log_info "Old backups cleaned up"
else
    log_error "Backup failed!"
    exit 1
fi
```

Create `scripts/restore-database.sh`:

```bash
#!/bin/bash

set -e

DB_URL="${DATABASE_URL:-postgresql://rustmart_user:rustmart_pass@localhost/rustmart}"
BACKUP_DIR="./backups"

if [ -z "$1" ]; then
    echo "Available backups:"
    ls -lh ${BACKUP_DIR}/rustmart_*.sql
    echo ""
    echo "Usage: $0 <backup_file>"
    exit 1
fi

BACKUP_FILE="$1"

if [ ! -f "$BACKUP_FILE" ]; then
    echo "Error: Backup file not found: $BACKUP_FILE"
    exit 1
fi

echo "Restoring from: $BACKUP_FILE"
psql "$DB_URL" < "$BACKUP_FILE"

echo "✓ Database restored successfully"
```

## Step 8: Error Handling Best Practices

```bash
#!/bin/bash

# Exit on error
set -e

# Exit on undefined variable
set -u

# Pipe failures fail the script
set -o pipefail

# Trap errors
trap 'echo "Error on line $LINENO"; exit 1' ERR

# Cleanup on exit
cleanup() {
    echo "Cleaning up..."
    # Remove temp files, etc.
}
trap cleanup EXIT

# Function with error handling
safe_curl() {
    local url=$1
    local max_retries=3
    local retry=0
    
    while [ $retry -lt $max_retries ]; do
        if curl -s -f "$url" > /dev/null; then
            return 0
        fi
        
        ((retry++))
        echo "Retry $retry/$max_retries..."
        sleep 2
    done
    
    echo "Failed after $max_retries attempts"
    return 1
}
```

## Key Bash Concepts Summary

| Feature | Syntax | Example |
|---------|--------|---------|
| Variables | `VAR=value` | `NAME="Terry"` |
| Command substitution | `$(command)` | `DATE=$(date)` |
| Conditionals | `if [ condition ]` | `if [ $x -gt 5 ]` |
| Loops | `for i in {1..10}` | `for f in *.txt` |
| Functions | `function_name() { }` | `hello() { echo "hi"; }` |
| Arrays | `ARR=(a b c)` | `echo ${ARR[0]}` |
| Exit on error | `set -e` | At top of script |
| Default values | `${VAR:-default}` | `PORT=${PORT:-8001}` |

## Comparison Operators

### Numeric
- `-eq`: equal
- `-ne`: not equal
- `-gt`: greater than
- `-lt`: less than
- `-ge`: greater or equal
- `-le`: less or equal

### String
- `=`: equal
- `!=`: not equal
- `-z`: empty string
- `-n`: non-empty string

### Files
- `-f`: file exists
- `-d`: directory exists
- `-r`: readable
- `-w`: writable
- `-x`: executable

## Challenges

1. **Add CSV export**: Create script that exports products to CSV
2. **Add data validation**: Check for duplicate product names before seeding
3. **Add progress bar**: Show visual progress during seeding
4. **Add logging to file**: Write all operations to a log file with timestamps
5. **Add interactive mode**: Menu-driven script with multiple options

<details>
<summary>Challenge 1 Solution: CSV Export</summary>

```bash
#!/bin/bash

API_BASE_URL="${API_BASE_URL:-http://localhost:8001}"
OUTPUT_FILE="${1:-products.csv}"

# Get all products and convert to CSV
curl -s "${API_BASE_URL}/products" | \
    jq -r '["ID","Name","Price","Inventory"], 
           (.[] | [.id, .name, .price, .inventory_count]) | 
           @csv' > "$OUTPUT_FILE"

echo "✓ Exported to $OUTPUT_FILE"
```

</details>

## Next Steps

In **Lesson 17**, you'll learn to build load testing tools to stress-test RustMart and measure performance!

## Official Documentation

- [Bash Manual](https://www.gnu.org/software/bash/manual/)
- [jq Manual](https://stedolan.github.io/jq/manual/)
- [ShellCheck](https://www.shellcheck.net/) - Bash linter
- [Bash Guide for Beginners](https://tldp.org/LDP/Bash-Beginners-Guide/html/)
- [Advanced Bash-Scripting Guide](https://tldp.org/LDP/abs/html/)

# Lesson 16.5: Deployment Automation - From Local to Linux Server

## Overview
Build deployment automation scripts to move RustMart from your local macOS development environment to a remote Linux server. This lesson bridges local development and CI/CD, teaching you the foundational automation that powers production deployments.

By the end of this lesson, you'll have:
- Deployment script to build and transfer Rust binaries to a Linux server
- Pre-deployment validation and health checks
- Automated rollback on failure
- Environment-specific configuration management
- Foundation for GitHub Actions CI/CD pipeline (Lesson 23)

## Why Manual Deployment Scripts First?

Before jumping to full CI/CD pipelines, you need to:
- **Understand the mechanics**: What actually happens during deployment?
- **Debug faster**: Scripts are easier to test and troubleshoot than CI/CD
- **Build incrementally**: Start simple, add complexity gradually
- **Have a fallback**: When CI/CD breaks, manual scripts save the day

**Career impact**: SREs who understand both manual and automated deployments are invaluable during incidents.

## Prerequisites

You'll need:
- A Linux server (AWS EC2, DigitalOcean, your own VM)
- SSH access to the server
- Your SSH key configured: `~/.ssh/id_rsa` or `~/.ssh/id_ed25519`

## Architecture Overview

```
┌─────────────────┐         ┌──────────────────┐         ┌────────────────┐
│   macOS Dev     │         │  Deployment      │         │  Linux Server  │
│   Environment   │────────▶│  Script          │────────▶│  (Production)  │
│                 │         │  (deploy.sh)     │         │                │
└─────────────────┘         └──────────────────┘         └────────────────┘
      │                              │                            │
      │ 1. Build binary             │                            │
      │ 2. Run tests                │                            │
      │                              │ 3. Transfer binary        │
      │                              │ 4. Stop old service       │
      │                              │ 5. Start new service      │
      │                              │ 6. Health check           │
      │                              │ 7. Rollback if fail       │
```

## Step 1: Server Preparation Script

First, prepare your Linux server. Create `scripts/setup-server.sh`:

```bash
#!/bin/bash

# Setup script for Linux server - run this once on the server

set -e

SERVER_USER="${SERVER_USER:-rustmart}"
APP_DIR="/opt/rustmart"
LOG_DIR="/var/log/rustmart"
SERVICE_NAME="product-service"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "Please run as root (sudo)"
    exit 1
fi

log_info "Setting up RustMart deployment environment..."

# Create application user
if ! id "$SERVER_USER" &>/dev/null; then
    log_info "Creating user: $SERVER_USER"
    useradd -r -s /bin/bash -d "$APP_DIR" "$SERVER_USER"
else
    log_warn "User $SERVER_USER already exists"
fi

# Create directories
log_info "Creating directories..."
mkdir -p "$APP_DIR/bin"
mkdir -p "$APP_DIR/config"
mkdir -p "$APP_DIR/backups"
mkdir -p "$LOG_DIR"

# Set permissions
chown -R "$SERVER_USER:$SERVER_USER" "$APP_DIR"
chown -R "$SERVER_USER:$SERVER_USER" "$LOG_DIR"

# Create systemd service
log_info "Creating systemd service..."
cat > "/etc/systemd/system/${SERVICE_NAME}.service" <<EOF
[Unit]
Description=RustMart Product Service
After=network.target

[Service]
Type=simple
User=$SERVER_USER
WorkingDirectory=$APP_DIR
ExecStart=$APP_DIR/bin/product-service
Restart=on-failure
RestartSec=5s

# Environment
Environment="RUST_LOG=info"
Environment="DATABASE_URL=postgres://rustmart:password@localhost/rustmart"

# Logging
StandardOutput=append:$LOG_DIR/product-service.log
StandardError=append:$LOG_DIR/product-service.error.log

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd
systemctl daemon-reload

log_info "✓ Server setup complete!"
log_info "Application directory: $APP_DIR"
log_info "Log directory: $LOG_DIR"
log_info "Systemd service: ${SERVICE_NAME}.service"
echo ""
log_info "Next steps:"
echo "  1. Run deployment script from your local machine"
echo "  2. Start service: sudo systemctl start ${SERVICE_NAME}"
echo "  3. Enable on boot: sudo systemctl enable ${SERVICE_NAME}"
```

**Run this once on your server:**
```bash
# Copy to server
scp scripts/setup-server.sh user@your-server:/tmp/

# SSH and run
ssh user@your-server
sudo bash /tmp/setup-server.sh
```

## Step 2: Basic Deployment Script

Create `scripts/deploy.sh`:

```bash
#!/bin/bash

# RustMart Deployment Script
# Deploys product-service to a Linux server

set -e  # Exit on error

# ============================================================================
# Configuration
# ============================================================================

# Server details
SERVER_HOST="${SERVER_HOST:-your-server.example.com}"
SERVER_USER="${SERVER_USER:-rustmart}"
SERVER_PORT="${SERVER_PORT:-22}"

# Paths
LOCAL_BIN="target/release/product-service"
REMOTE_APP_DIR="/opt/rustmart"
REMOTE_BIN_DIR="$REMOTE_APP_DIR/bin"
REMOTE_BACKUP_DIR="$REMOTE_APP_DIR/backups"

# Service
SERVICE_NAME="product-service"

# Build settings
BUILD_TARGET="x86_64-unknown-linux-gnu"  # Linux target

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# ============================================================================
# Helper Functions
# ============================================================================

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo ""
    echo -e "${BLUE}===> $1${NC}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# SSH wrapper
ssh_exec() {
    ssh -p "$SERVER_PORT" "${SERVER_USER}@${SERVER_HOST}" "$@"
}

# SCP wrapper
scp_upload() {
    scp -P "$SERVER_PORT" "$1" "${SERVER_USER}@${SERVER_HOST}:$2"
}

# ============================================================================
# Validation Functions
# ============================================================================

validate_environment() {
    log_step "Validating environment..."
    
    # Check SSH connectivity
    if ! ssh_exec "echo 'SSH connection OK'" &>/dev/null; then
        log_error "Cannot connect to $SERVER_HOST"
        log_info "Check: ssh ${SERVER_USER}@${SERVER_HOST}"
        exit 1
    fi
    log_info "✓ SSH connection successful"
    
    # Check if cross-compilation target is installed
    if ! rustup target list | grep -q "$BUILD_TARGET (installed)"; then
        log_warn "Cross-compilation target not installed: $BUILD_TARGET"
        log_info "Installing..."
        rustup target add "$BUILD_TARGET"
    fi
    log_info "✓ Build target ready: $BUILD_TARGET"
    
    # Check remote directories
    if ! ssh_exec "[ -d $REMOTE_APP_DIR ]"; then
        log_error "Remote app directory doesn't exist: $REMOTE_APP_DIR"
        log_info "Run setup-server.sh on the server first"
        exit 1
    fi
    log_info "✓ Remote directories exist"
}

# ============================================================================
# Build Functions
# ============================================================================

build_binary() {
    log_step "Building binary for Linux..."
    
    # Clean previous build
    log_info "Cleaning previous build..."
    cargo clean -p product-service
    
    # Build for Linux
    log_info "Cross-compiling for $BUILD_TARGET..."
    cargo build --release --target "$BUILD_TARGET" --package product-service
    
    # Verify binary
    LOCAL_BIN="target/${BUILD_TARGET}/release/product-service"
    if [ ! -f "$LOCAL_BIN" ]; then
        log_error "Build failed - binary not found at $LOCAL_BIN"
        exit 1
    fi
    
    # Show binary size
    local size=$(du -h "$LOCAL_BIN" | cut -f1)
    log_info "✓ Build complete - Binary size: $size"
}

run_tests() {
    log_step "Running tests..."
    
    if cargo test --package product-service --release; then
        log_info "✓ All tests passed"
    else
        log_error "Tests failed!"
        read -p "Continue deployment anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# ============================================================================
# Deployment Functions
# ============================================================================

backup_current_binary() {
    log_step "Backing up current binary..."
    
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_name="product-service_${timestamp}"
    
    # Create backup on server
    ssh_exec "
        if [ -f $REMOTE_BIN_DIR/product-service ]; then
            cp $REMOTE_BIN_DIR/product-service $REMOTE_BACKUP_DIR/$backup_name
            echo 'Backup created: $backup_name'
        else
            echo 'No existing binary to backup'
        fi
    "
    
    log_info "✓ Backup complete"
}

stop_service() {
    log_step "Stopping service..."
    
    if ssh_exec "sudo systemctl is-active --quiet $SERVICE_NAME"; then
        ssh_exec "sudo systemctl stop $SERVICE_NAME"
        log_info "✓ Service stopped"
    else
        log_warn "Service was not running"
    fi
}

upload_binary() {
    log_step "Uploading binary..."
    
    # Upload to temp location first
    log_info "Transferring binary to server..."
    scp_upload "$LOCAL_BIN" "/tmp/product-service"
    
    # Move to final location
    ssh_exec "
        chmod +x /tmp/product-service
        sudo mv /tmp/product-service $REMOTE_BIN_DIR/product-service
        sudo chown $SERVER_USER:$SERVER_USER $REMOTE_BIN_DIR/product-service
    "
    
    log_info "✓ Binary uploaded and permissions set"
}

start_service() {
    log_step "Starting service..."
    
    ssh_exec "sudo systemctl start $SERVICE_NAME"
    sleep 2  # Give service time to start
    
    if ssh_exec "sudo systemctl is-active --quiet $SERVICE_NAME"; then
        log_info "✓ Service started successfully"
    else
        log_error "Service failed to start!"
        return 1
    fi
}

health_check() {
    log_step "Running health check..."
    
    local max_attempts=10
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        log_info "Attempt $attempt/$max_attempts..."
        
        if ssh_exec "curl -f -s http://localhost:8001/health" &>/dev/null; then
            log_info "✓ Health check passed!"
            return 0
        fi
        
        sleep 2
        ((attempt++))
    done
    
    log_error "Health check failed after $max_attempts attempts"
    return 1
}

rollback() {
    log_step "Rolling back to previous version..."
    
    local latest_backup=$(ssh_exec "ls -t $REMOTE_BACKUP_DIR | head -n1")
    
    if [ -z "$latest_backup" ]; then
        log_error "No backup found for rollback!"
        return 1
    fi
    
    log_info "Restoring from backup: $latest_backup"
    ssh_exec "
        sudo systemctl stop $SERVICE_NAME
        sudo cp $REMOTE_BACKUP_DIR/$latest_backup $REMOTE_BIN_DIR/product-service
        sudo systemctl start $SERVICE_NAME
    "
    
    log_info "✓ Rollback complete"
}

show_logs() {
    log_step "Recent logs..."
    ssh_exec "sudo journalctl -u $SERVICE_NAME -n 20 --no-pager"
}

# ============================================================================
# Main Deployment Flow
# ============================================================================

deploy() {
    log_info "RustMart Deployment Script"
    log_info "Target: ${SERVER_USER}@${SERVER_HOST}"
    echo ""
    
    # Pre-deployment
    validate_environment
    build_binary
    run_tests
    
    # Deployment
    backup_current_binary
    stop_service
    upload_binary
    
    # Post-deployment
    if start_service && health_check; then
        log_info ""
        log_info "════════════════════════════════════════"
        log_info "  ✓ Deployment successful!"
        log_info "════════════════════════════════════════"
        show_logs
    else
        log_error ""
        log_error "════════════════════════════════════════"
        log_error "  ✗ Deployment failed!"
        log_error "════════════════════════════════════════"
        
        read -p "Rollback to previous version? (Y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            rollback
            if health_check; then
                log_info "✓ Rollback successful"
            else
                log_error "✗ Rollback failed - manual intervention required"
            fi
        fi
        
        exit 1
    fi
}

# ============================================================================
# Script Entry Point
# ============================================================================

# Parse command line arguments
case "${1:-deploy}" in
    deploy)
        deploy
        ;;
    validate)
        validate_environment
        ;;
    build)
        build_binary
        ;;
    rollback)
        rollback
        ;;
    logs)
        show_logs
        ;;
    *)
        echo "Usage: $0 {deploy|validate|build|rollback|logs}"
        exit 1
        ;;
esac
```

## Step 3: Configuration Management

Create `.env.production` for server-specific config:

```bash
# .env.production - Configuration for production server

# Server connection
SERVER_HOST=your-server.example.com
SERVER_USER=rustmart
SERVER_PORT=22

# Application settings
DATABASE_URL=postgres://rustmart:password@localhost/rustmart
RUST_LOG=info
TRACING_BACKEND=instana
INSTANA_AGENT_HOST=localhost

# Service ports
HTTP_PORT=8001
```

Update `deploy.sh` to load config:

```bash
# At the top of deploy.sh, after set -e

# Load environment-specific configuration
ENV_FILE="${ENV_FILE:-.env.production}"
if [ -f "$ENV_FILE" ]; then
    log_info "Loading configuration from $ENV_FILE"
    export $(grep -v '^#' "$ENV_FILE" | xargs)
fi
```

## Step 4: Usage Examples

Make the script executable:

```bash
chmod +x scripts/deploy.sh scripts/setup-server.sh
```

### Basic Deployment

```bash
cd ~/code/rustmart

# Deploy to production
./scripts/deploy.sh

# Or specify environment
ENV_FILE=.env.staging ./scripts/deploy.sh
```

### Validation Only

```bash
# Check connectivity and environment
./scripts/deploy.sh validate
```

### Build Only (for testing)

```bash
# Just build the Linux binary
./scripts/deploy.sh build
```

### View Logs

```bash
# Check service logs
./scripts/deploy.sh logs
```

### Manual Rollback

```bash
# Rollback to previous version
./scripts/deploy.sh rollback
```

## Step 5: Multi-Service Deployment

For deploying multiple services, create `scripts/deploy-all.sh`:

```bash
#!/bin/bash

set -e

SERVICES=("product-service" "order-service" "inventory-service")

for service in "${SERVICES[@]}"; do
    echo ""
    echo "════════════════════════════════════════"
    echo "  Deploying $service"
    echo "════════════════════════════════════════"
    
    SERVICE_NAME="$service" ./scripts/deploy.sh
    
    if [ $? -ne 0 ]; then
        echo "Deployment of $service failed!"
        read -p "Continue with remaining services? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
done

echo ""
echo "✓ All services deployed!"
```

## Step 6: Dry Run Mode

Add dry-run capability to test deployments:

```bash
# Add near the top of deploy.sh
DRY_RUN="${DRY_RUN:-false}"

# Wrap deployment functions
if [ "$DRY_RUN" = "true" ]; then
    ssh_exec() { echo "[DRY RUN] Would execute: ssh $*"; }
    scp_upload() { echo "[DRY RUN] Would upload: $1 to $2"; }
fi
```

Usage:

```bash
# Test deployment without making changes
DRY_RUN=true ./scripts/deploy.sh
```

## Step 7: Integration with CI/CD (Preview)

Your manual script becomes the foundation for GitHub Actions (Lesson 23):

```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    tags:
      - 'v*'

jobs:
  deploy:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-unknown-linux-gnu
      
      - name: Configure SSH
        run: |
          mkdir -p ~/.ssh
          echo "${{ secrets.SSH_PRIVATE_KEY }}" > ~/.ssh/id_rsa
          chmod 600 ~/.ssh/id_rsa
          ssh-keyscan -H ${{ secrets.SERVER_HOST }} >> ~/.ssh/known_hosts
      
      - name: Deploy
        env:
          SERVER_HOST: ${{ secrets.SERVER_HOST }}
          SERVER_USER: ${{ secrets.SERVER_USER }}
        run: ./scripts/deploy.sh
```

## Troubleshooting

### Common Issues

**1. Cross-compilation fails**
```bash
# Install linker for cross-compilation
brew install FiloSottile/musl-cross/musl-cross

# Or use Docker to build
docker run --rm -v "$(pwd)":/app -w /app rust:latest cargo build --release --target x86_64-unknown-linux-gnu
```

**2. Service won't start**
```bash
# Check logs on server
ssh user@server sudo journalctl -u product-service -f

# Check binary dependencies
ssh user@server ldd /opt/rustmart/bin/product-service
```

**3. Health check fails**
```bash
# Test endpoint manually
ssh user@server curl -v http://localhost:8001/health

# Check if port is listening
ssh user@server sudo netstat -tlnp | grep 8001
```

## Best Practices

1. **Always backup before deployment**
   - Keep last 5 versions: `find $BACKUP_DIR -type f | sort -r | tail -n +6 | xargs rm`

2. **Use health checks**
   - Verify service is actually working, not just running

3. **Graceful degradation**
   - Automatic rollback on failure
   - Manual intervention as fallback

4. **Logging**
   - Log every step with timestamps
   - Separate stdout/stderr logs

5. **Secrets management**
   - Never commit secrets to git
   - Use `.env` files (git-ignored)
   - In production: use secret managers (AWS Secrets Manager, HashiCorp Vault)

## Key Takeaways

1. **Understand before automating**: Manual scripts teach you what actually happens
2. **Incremental complexity**: Start simple, add features as needed
3. **Rollback capability**: Always have a way back
4. **Validation first**: Check environment before making changes
5. **Foundation for CI/CD**: These scripts become your pipeline steps

## Next Steps

- **Lesson 23**: Convert these scripts to GitHub Actions workflows
- **Lesson 14**: Deploy to Kubernetes instead of systemd
- **Lesson 20**: Add security hardening to deployment process

## Practice Exercise

Deploy your product-service to a Linux server:

1. Spin up a DigitalOcean droplet ($6/month) or AWS EC2 instance
2. Run `setup-server.sh` to prepare the server
3. Configure `.env.production` with your server details
4. Run `./scripts/deploy.sh` to deploy
5. Make a code change and deploy again
6. Simulate a failure and test rollback

**Bonus**: Add monitoring to your deployment:
- Send Slack notification on deployment
- Report deployment time to Prometheus
- Integrate with Instana for deployment markers

---

**Why this matters for IBM**: Understanding deployment mechanics makes you better at debugging CI/CD issues, designing reliable pipelines, and explaining architecture decisions to stakeholders. This hands-on knowledge separates senior engineers from junior ones.

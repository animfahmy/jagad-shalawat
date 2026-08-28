#!/usr/bin/env bash
# =================================================================
# Jagad Shalawat Blog — Deployment Script
# Deploy Rust binary to GCP VPS
# =================================================================

set -Eeuo pipefail

readonly REMOTE_USER="${REMOTE_USER:-root}"
readonly REMOTE_HOST="${REMOTE_HOST:-your-gcp-ip}"
readonly REMOTE_DIR="/var/www/jagad-shalawat"
readonly BINARY_NAME="jagad-shalawat"
readonly SERVICE_NAME="jagad-shalawat-blog"

echo "🚀 Deploying Jagad Shalawat Blog to ${REMOTE_HOST}..."

# Check if binary exists
if [ ! -f "target/x86_64-unknown-linux-gnu/release/${BINARY_NAME}" ]; then
    echo "❌ Binary not found. Run build-linux.sh first."
    exit 1
fi

# Create remote directory if needed
ssh "${REMOTE_USER}@${REMOTE_HOST}" "mkdir -p ${REMOTE_DIR}/src/templates ${REMOTE_DIR}/src/static ${REMOTE_DIR}/migrations ${REMOTE_DIR}/seeds"

# Upload binary
echo "📦 Uploading binary..."
scp "target/x86_64-unknown-linux-gnu/release/${BINARY_NAME}" "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/${BINARY_NAME}.new"

# Upload templates
echo "📄 Uploading templates..."
scp -r src/templates/* "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/src/templates/"

# Upload static assets
echo "🎨 Uploading static assets..."
scp -r src/static/* "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/src/static/"

# Upload migrations and seeds
echo "🗄️ Uploading migrations and seeds..."
scp -r migrations/* "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/migrations/"
scp -r seeds/* "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/seeds/"

# Upload .env if it doesn't exist on remote
ssh "${REMOTE_USER}@${REMOTE_HOST}" "[ -f ${REMOTE_DIR}/.env ] || echo 'BIND_ADDRESS=127.0.0.1:8080' > ${REMOTE_DIR}/.env"

# Atomic swap and restart
echo "🔄 Swapping binary and restarting service..."
ssh "${REMOTE_USER}@${REMOTE_HOST}" << EOF
    cd ${REMOTE_DIR}
    chmod +x ${BINARY_NAME}.new
    mv ${BINARY_NAME}.new ${BINARY_NAME}
    
    # Restart service
    if systemctl is-active --quiet ${SERVICE_NAME}; then
        systemctl restart ${SERVICE_NAME}
        echo "♻️ Service restarted"
    else
        echo "⚠️ Service not running. Start with: systemctl start ${SERVICE_NAME}"
    fi
    
    # Wait and check
    sleep 2
    if systemctl is-active --quiet ${SERVICE_NAME}; then
        echo "✅ Service is running"
    else
        echo "❌ Service failed to start. Check: journalctl -u ${SERVICE_NAME} -n 20"
        exit 1
    fi
EOF

echo "✅ Deployment complete!"

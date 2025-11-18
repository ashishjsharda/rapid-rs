#!/bin/bash

# rapid-rs Launch Helper Script
# This script helps you launch rapid-rs to the world!

set -e

echo "🚀 rapid-rs Launch Helper"
echo "=========================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must run this from the rapid-rs root directory"
    exit 1
fi

# Function to ask yes/no questions
ask() {
    local prompt="$1"
    local response
    read -p "$prompt (y/n): " response
    case "$response" in
        [yY]|[yY][eE][sS]) return 0 ;;
        *) return 1 ;;
    esac
}

echo "Step 1: Initialize Git Repository"
if ask "Initialize git and commit?"; then
    git init
    git add .
    git commit -m "Initial commit - rapid-rs v0.1.0 🚀"
    echo "✅ Git initialized and committed"
else
    echo "⏭️  Skipping git initialization"
fi
echo ""

echo "Step 2: Test Build"
if ask "Run cargo build to test everything compiles?"; then
    cargo build
    echo "✅ Build successful"
else
    echo "⏭️  Skipping build test"
fi
echo ""

echo "Step 3: Test Example"
if ask "Test the example (this will start a server)?"; then
    echo "Starting server... Press Ctrl+C to stop"
    cd examples/rest-api
    timeout 10s cargo run || true
    cd ../..
    echo "✅ Example tested"
else
    echo "⏭️  Skipping example test"
fi
echo ""

echo "Step 4: Push to GitHub"
echo "Go to: https://github.com/new"
echo "Create repository: rapid-rs"
echo "DON'T initialize with README"
echo ""
if ask "Ready to push to GitHub?"; then
    read -p "Enter your GitHub username (default: ashishjsharda): " github_user
    github_user=${github_user:-ashishjsharda}
    
    git remote add origin "https://github.com/${github_user}/rapid-rs.git"
    git branch -M main
    git push -u origin main
    echo "✅ Pushed to GitHub"
    echo "🌐 Repository: https://github.com/${github_user}/rapid-rs"
else
    echo "⏭️  Skipping GitHub push"
fi
echo ""

echo "Step 5: Social Media"
echo ""
echo "Ready to share? Here are your posts:"
echo ""
echo "📱 Twitter/X (copy this):"
echo "----------------------------------------"
cat << 'EOF'
🚀 Just launched rapid-rs - zero-config web framework for Rust!

✅ Type-safe APIs with auto docs
✅ One command: rapid new myapi  
✅ Built on Axum
✅ FastAPI DX + Spring Boot conventions

Stop wiring boilerplate, start shipping.

https://github.com/ashishjsharda/rapid-rs

#rustlang #webdev #opensource
EOF
echo "----------------------------------------"
echo ""

echo "📱 Reddit r/rust:"
echo "See MARKETING.md for the full post"
echo ""

echo "📱 LinkedIn:"
echo "See MARKETING.md for the long post"
echo ""

if ask "Open marketing guide in browser?"; then
    if command -v open &> /dev/null; then
        open MARKETING.md
    elif command -v xdg-open &> /dev/null; then
        xdg-open MARKETING.md
    else
        echo "Please open MARKETING.md manually"
    fi
fi
echo ""

echo "🎉 Launch Complete!"
echo ""
echo "Next Steps:"
echo "1. Post on Twitter/X NOW"
echo "2. Post on LinkedIn (within 1 hour)"
echo "3. Post on Reddit r/rust (within 2 hours)"
echo "4. Monitor GitHub for issues/stars"
echo ""
echo "You've got this! 🚀"

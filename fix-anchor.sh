#!/usr/bin/env zsh
# Fix: standalone anchor-cli 0.31.1 in ~/.cargo/bin shadows avm-managed versions.
set -euo pipefail

VERSION="${1:-1.2.2}"

echo "==> Removing standalone anchor-cli (if installed via cargo)..."
if cargo install --list | grep -q '^anchor-cli '; then
  cargo uninstall anchor-cli
else
  echo "    anchor-cli not installed via cargo, skipping."
fi

echo "==> Installing latest avm..."
cargo install --git https://github.com/solana-foundation/anchor avm --force --locked

echo "==> Selecting anchor $VERSION..."
avm use "$VERSION"
hash -r

# Make sure avm's shim dir is on PATH ahead of cargo's, if it exists
if [[ -d "$HOME/.avm/bin" ]] && ! grep -q '.avm/bin' "$HOME/.zshrc" 2>/dev/null; then
  if ! command -v anchor >/dev/null 2>&1; then
    echo "==> Adding ~/.avm/bin to PATH in ~/.zshrc"
    echo 'export PATH="$HOME/.avm/bin:$PATH"' >> "$HOME/.zshrc"
    export PATH="$HOME/.avm/bin:$PATH"
  fi
fi

echo "==> Result:"
which -a anchor
anchor --version
echo
echo "Done. Open a new terminal (or run: source ~/.zshrc) to pick up any PATH change."

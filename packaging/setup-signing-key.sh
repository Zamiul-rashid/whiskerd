#!/usr/bin/env bash
# Maintainer helper: create the package-signing key used by .github/workflows/release.yml.
#
# - Generates an RSA 4096 signing key (no passphrase, valid 5 years) in a throwaway keyring.
# - Stores the private key ONLY as the GitHub Actions secret PKG_GPG_PRIVATE_KEY.
# - Writes the public key to packaging/repo/whiskerd.asc (commit it).
# - Deletes the throwaway keyring.
#
# Usage: packaging/setup-signing-key.sh [owner/repo]   (default: Zamiul-rashid/whiskerd)
set -euo pipefail
cd "$(dirname "$0")/.."

REPO="${1:-Zamiul-rashid/whiskerd}"
UID_STR="whiskerd packages <Zamiul-rashid@users.noreply.github.com>"

GNUPGHOME="$(mktemp -d)"
export GNUPGHOME
trap 'rm -rf "$GNUPGHOME"' EXIT

gpg --batch --quiet --pinentry-mode loopback --passphrase '' \
    --quick-gen-key "$UID_STR" rsa4096 sign 5y
FPR="$(gpg --list-secret-keys --with-colons | awk -F: '/^fpr/ {print $10; exit}')"

gpg --armor --export-secret-keys "$FPR" | gh secret set PKG_GPG_PRIVATE_KEY -R "$REPO"
gpg --armor --export "$FPR" > packaging/repo/whiskerd.asc

echo "Signing key $FPR stored as secret PKG_GPG_PRIVATE_KEY in $REPO."
echo "Public key written to packaging/repo/whiskerd.asc."

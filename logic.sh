#!/usr/bin/env bash
# logic.sh — reasoning as infrastructure
# Pre-flight script for Freenet/IPFRS contracts

set -e  # fail fast on any error

echo "🔎 Running reasoning pre-flight checks..."

# 1. Premise checks
premise_contract="true"
premise_tests="true"

if [ "$premise_contract" != "true" ] || [ "$premise_tests" != "true" ]; then
  echo "❌ Premises incomplete — stop build"
  exit 1
else
  echo "✅ Premises satisfied — proceeding"
fi

# 2. Inspect contract
echo "🔍 Inspecting contract..."
freenet inspect target/wasm32-unknown-unknown/release/my_contract.wasm \
  || { echo "❌ Bad logic upstream — inspect failed"; exit 1; }

# 3. Verify merge laws
echo "🔍 Verifying merge laws..."
freenet verify-merge target/wasm32-unknown-unknown/release/my_contract.wasm \
  || { echo "❌ Merge inconsistency detected"; exit 1; }

# 4. Run tests
echo "🧪 Running Freenet tests..."
freenet test \
  || { echo "❌ Tests failed — stop build"; exit 1; }

# 5. Semantic tagging (IPFRS tensorlogic)
echo "🔍 Running tensorlogic semantic evaluation..."
tensorlogic_eval "rendered scene" \
  || { echo "❌ Semantic index missing"; exit 1; }

# 6. Diagnostics
echo "📊 Gathering diagnostics..."
freenet diagnostics

echo "✅ Pre-flight checks passed — safe to publish"

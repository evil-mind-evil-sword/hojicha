#!/bin/bash

echo "Fixing imports in examples..."

# Fix imports in all example files
find examples -name "*.rs" -exec sed -i '' \
  -e 's/use hojicha::{/use hojicha_core::{/g' \
  -e 's/use hojicha::core::/use hojicha_core::core::/g' \
  -e 's/use hojicha::commands/use hojicha_core::commands/g' \
  -e 's/use hojicha::event::/use hojicha_core::event::/g' \
  -e 's/use hojicha::error::/use hojicha_core::error::/g' \
  -e 's/use hojicha::program::/use hojicha_runtime::program::/g' \
  -e 's/use hojicha::components::/use hojicha_pearls::components::/g' \
  -e 's/use hojicha::style::/use hojicha_pearls::style::/g' \
  -e 's/use hojicha::logging/use hojicha_core::logging/g' \
  -e 's/use hojicha::testing::/use hojicha_core::testing::/g' \
  {} \;

# Add proper imports where needed
for file in examples/*.rs; do
  echo "Processing $file..."
  
  # Check if file uses components
  if grep -q "components::" "$file"; then
    # Make sure hojicha_pearls is imported
    if ! grep -q "use hojicha_pearls" "$file"; then
      sed -i '' '1s/^/use hojicha_pearls;\n/' "$file"
    fi
  fi
  
  # Check if file uses runtime features
  if grep -q "program::\|Program\|ProgramOptions" "$file"; then
    # Make sure hojicha_runtime is imported
    if ! grep -q "use hojicha_runtime" "$file"; then
      sed -i '' '1s/^/use hojicha_runtime;\n/' "$file"
    fi
  fi
done

echo "Done!"
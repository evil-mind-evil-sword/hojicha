#!/bin/bash

echo "Fixing test imports to use new crate structure..."

# Fix all test files in tests/ directory
for file in tests/*.rs; do
    if [ -f "$file" ]; then
        echo "Fixing $file..."
        
        # Replace hojicha imports with proper crate imports
        sed -i '' \
            -e 's/use hojicha::{/use hojicha_core::{/g' \
            -e 's/use hojicha::core::/use hojicha_core::core::/g' \
            -e 's/use hojicha::commands/use hojicha_core::commands/g' \
            -e 's/use hojicha::event::/use hojicha_core::event::/g' \
            -e 's/use hojicha::error::/use hojicha_core::error::/g' \
            -e 's/use hojicha::fallible::/use hojicha_core::fallible::/g' \
            -e 's/use hojicha::logging/use hojicha_core::logging/g' \
            -e 's/use hojicha::testing::/use hojicha_core::testing::/g' \
            -e 's/use hojicha::prelude::/use hojicha_core::prelude::/g' \
            -e 's/hojicha::Result/hojicha_core::Result/g' \
            -e 's/hojicha::Error/hojicha_core::Error/g' \
            -e 's/use hojicha::program::/use hojicha_runtime::program::/g' \
            -e 's/hojicha::program::/hojicha_runtime::program::/g' \
            -e 's/use hojicha::components::/use hojicha_pearls::components::/g' \
            -e 's/use hojicha::style::/use hojicha_pearls::style::/g' \
            -e 's/hojicha::components::/hojicha_pearls::components::/g' \
            -e 's/hojicha::style::/hojicha_pearls::style::/g' \
            "$file"
            
        # Handle complex imports that might span multiple lines
        # If file has program imports, ensure hojicha_runtime is imported
        if grep -q "program::" "$file"; then
            # Check if hojicha_runtime is already imported
            if ! grep -q "use hojicha_runtime" "$file"; then
                # Add the import at the top after the first use statement
                sed -i '' '0,/^use /{s/^use /use hojicha_runtime;\nuse /}' "$file"
            fi
        fi
        
        # If file has component or style imports, ensure hojicha_pearls is imported  
        if grep -q -E "(components::|style::)" "$file"; then
            if ! grep -q "use hojicha_pearls" "$file"; then
                sed -i '' '0,/^use /{s/^use /use hojicha_pearls;\nuse /}' "$file"
            fi
        fi
    fi
done

echo "Test imports fixed!"
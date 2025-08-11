#!/bin/bash

echo "Fixing imports in runtime crate..."

# Fix imports in runtime/src
find runtime/src -name "*.rs" -exec sed -i '' \
  -e 's/use crate::event::/use hojicha_core::event::/g' \
  -e 's/use crate::core::/use hojicha_core::core::/g' \
  -e 's/use crate::error::/use hojicha_core::error::/g' \
  -e 's/use crate::Event/use hojicha_core::Event/g' \
  -e 's/use crate::Cmd/use hojicha_core::Cmd/g' \
  -e 's/use crate::Model/use hojicha_core::Model/g' \
  -e 's/use crate::Message/use hojicha_core::Message/g' \
  {} \;

echo "Fixing imports in pearls crate..."

# Fix imports in pearls/src  
find pearls/src -name "*.rs" -exec sed -i '' \
  -e 's/use crate::event::/use hojicha_core::event::/g' \
  -e 's/use crate::core::/use hojicha_core::core::/g' \
  -e 's/use crate::error::/use hojicha_core::error::/g' \
  -e 's/use hojicha::event::/use hojicha_core::event::/g' \
  -e 's/use hojicha::core::/use hojicha_core::core::/g' \
  {} \;

echo "Done!"
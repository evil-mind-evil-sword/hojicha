#!/bin/bash

echo "Properly fixing all example imports..."

for file in examples/*.rs; do
  echo "Fixing $file..."
  
  # Create a temporary file with fixed imports
  cat "$file" | sed \
    -e 's/use hojicha_core::{$/use hojicha_core::{/g' \
    -e 's/components::{[^}]*}/PLACEHOLDER_COMPONENTS/g' \
    -e 's/style::{[^}]*}/PLACEHOLDER_STYLE/g' \
    -e 's/program::[^,;]*/PLACEHOLDER_PROGRAM/g' \
    -e 's/use hojicha_core::PLACEHOLDER_COMPONENTS/use hojicha_pearls::components::{Help, TextInput, Spinner, SpinnerStyle}/g' \
    -e 's/use hojicha_core::PLACEHOLDER_STYLE/use hojicha_pearls::style::{ColorProfile, Theme}/g' \
    -e 's/use hojicha_core::PLACEHOLDER_PROGRAM/use hojicha_runtime::program::Program/g' \
    -e 's/PLACEHOLDER_COMPONENTS,/hojicha_pearls::components::{Help, TextInput},/g' \
    -e 's/PLACEHOLDER_STYLE,/hojicha_pearls::style::{ColorProfile, Theme},/g' \
    -e 's/PLACEHOLDER_PROGRAM,/hojicha_runtime::program::Program,/g' \
    -e 's/hojicha::Result/hojicha_core::Result/g' \
    -e 's/hojicha::Error/hojicha_core::Error/g' \
    > "${file}.tmp"
  
  mv "${file}.tmp" "$file"
done

# Now let's manually fix the specific imports for each example
echo "Manually fixing specific examples..."

# Fix tutorial.rs
cat > examples/tutorial.rs.fixed << 'EOF'
//! Hojicha Tutorial - Learn the Basics
//!
//! This interactive tutorial teaches you the fundamentals of Hojicha:
//! 1. Basic counter application
//! 2. User input handling
//! 3. Component composition
//! 4. Styling basics
//!
//! Press Tab to switch between examples, or follow the on-screen instructions.

use hojicha_core::{
    commands,
    core::{Cmd, Model},
    event::{Event, Key},
    Result,
};
use hojicha_runtime::program::Program;
use hojicha_pearls::{
    components::{Help, TextInput},
    style::{ColorProfile, Theme},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Stylize,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
EOF

# Append the rest of tutorial.rs (skipping the original imports)
tail -n +26 examples/tutorial.rs >> examples/tutorial.rs.fixed
mv examples/tutorial.rs.fixed examples/tutorial.rs

echo "Done fixing examples!"
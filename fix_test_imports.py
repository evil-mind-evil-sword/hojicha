#!/usr/bin/env python3

import os
import re

def fix_test_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    
    # Remove any program imports from hojicha_core
    content = re.sub(r'use hojicha_core::program::[^;]+;', '', content)
    content = re.sub(r',\s*program::[^,}\]]+', '', content)
    
    # Check if file uses Program, ProgramOptions, or MouseMode
    needs_program = bool(re.search(r'\b(Program|ProgramOptions|MouseMode)\b', content))
    
    if needs_program:
        # Check if runtime import already exists
        has_runtime = bool(re.search(r'use hojicha_runtime::program', content))
        
        if not has_runtime:
            # Add runtime import after the last hojicha_core import
            import_line = 'use hojicha_runtime::program::{Program, ProgramOptions, MouseMode};'
            
            # Find a good place to insert it
            core_import = re.search(r'(use hojicha_core::[^;]+;)', content)
            if core_import:
                # Insert after the hojicha_core import
                pos = core_import.end()
                content = content[:pos] + '\n' + import_line + content[pos:]
    
    # Also check for async_handle, subscription, stream_builders
    if re.search(r'\b(init_async_bridge|subscribe|spawn_cancellable|AsyncBridge)\b', content):
        if not re.search(r'use hojicha_runtime::', content):
            # Add runtime imports
            runtime_imports = []
            if 'init_async_bridge' in content or 'AsyncBridge' in content:
                runtime_imports.append('use hojicha_runtime::async_handle::init_async_bridge;')
            if 'subscribe' in content:
                runtime_imports.append('use hojicha_runtime::subscription::subscribe;')
            if 'spawn_cancellable' in content:
                runtime_imports.append('use hojicha_runtime::async_handle::spawn_cancellable;')
            
            if runtime_imports:
                core_import = re.search(r'(use hojicha_core::[^;]+;)', content)
                if core_import:
                    pos = core_import.end()
                    content = content[:pos] + '\n' + '\n'.join(runtime_imports) + content[pos:]
    
    # Check for components and styles
    if re.search(r'\b(TextInput|Help|Spinner|Button|Modal|Tabs)\b', content):
        if not re.search(r'use hojicha_pearls::components', content):
            # Add pearls component import
            import_line = 'use hojicha_pearls::components::{TextInput, Help, Spinner, Button, Modal, Tabs};'
            core_import = re.search(r'(use hojicha_core::[^;]+;)', content)
            if core_import:
                pos = core_import.end()
                content = content[:pos] + '\n' + import_line + content[pos:]
    
    if re.search(r'\b(Theme|ColorProfile|Style)\b', content):
        if not re.search(r'use hojicha_pearls::style', content):
            # Add pearls style import
            import_line = 'use hojicha_pearls::style::{Theme, ColorProfile, Style};'
            core_import = re.search(r'(use hojicha_core::[^;]+;)', content)
            if core_import:
                pos = core_import.end()
                content = content[:pos] + '\n' + import_line + content[pos:]
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        print(f"Fixed: {filepath}")
        return True
    return False

# Fix all test files
test_dir = 'tests'
fixed_count = 0
for filename in os.listdir(test_dir):
    if filename.endswith('.rs'):
        filepath = os.path.join(test_dir, filename)
        if fix_test_file(filepath):
            fixed_count += 1

print(f"\nFixed {fixed_count} test files")
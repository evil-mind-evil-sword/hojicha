#!/usr/bin/env python3

import os
import re

def fix_test_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    
    # Remove any incorrect ProgramOptions in event import
    content = re.sub(r'(event::[^}]+), ProgramOptions}', r'\1}', content)
    
    # Remove any Program or ProgramOptions from hojicha_core imports
    content = re.sub(r', Program,', ',', content)
    content = re.sub(r', ProgramOptions,', ',', content)
    
    # Check if file needs runtime imports
    needs_runtime = bool(re.search(r'\b(Program|ProgramOptions|MouseMode)\b', content))
    
    if needs_runtime:
        # Check if runtime import already exists
        if not re.search(r'use hojicha_runtime::program', content):
            # Add runtime import after hojicha_core imports
            import_line = 'use hojicha_runtime::program::{Program, ProgramOptions, MouseMode};'
            
            # Find a good place to insert it
            core_import = re.search(r'(use hojicha_core::[^;]+;)', content)
            if core_import:
                pos = core_import.end()
                content = content[:pos] + '\n' + import_line + content[pos:]
    
    # Fix async imports if needed
    if 'init_async_bridge' in content:
        if not re.search(r'use hojicha_runtime::async_handle', content):
            content = re.sub(r'(use hojicha_runtime::program[^;]+;)', 
                           r'\1\nuse hojicha_runtime::async_handle::init_async_bridge;', content)
    
    if 'subscribe' in content and 'subscription::subscribe' not in content:
        if not re.search(r'use hojicha_runtime::subscription', content):
            content = re.sub(r'(use hojicha_runtime::program[^;]+;)', 
                           r'\1\nuse hojicha_runtime::subscription::subscribe;', content)
    
    # Fix component imports
    if re.search(r'\b(TextInput|Help|Spinner|List|Table|Viewport)\b', content):
        if not re.search(r'use hojicha_pearls::components', content):
            components = []
            if 'TextInput' in content: components.append('TextInput')
            if 'Help' in content: components.append('Help')
            if 'Spinner' in content: components.append('Spinner')
            if 'List' in content: components.append('List')
            if 'Table' in content: components.append('Table')
            if 'Viewport' in content: components.append('Viewport')
            
            if components:
                import_line = f"use hojicha_pearls::components::{{{', '.join(components)}}};"
                core_import = re.search(r'(use hojicha_core::[^;]+;)', content)
                if core_import:
                    pos = core_import.end()
                    content = content[:pos] + '\n' + import_line + content[pos:]
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        print(f"Fixed: {os.path.basename(filepath)}")
        return True
    return False

# Fix all behavioral tests
test_dir = 'tests/behavioral'
fixed_count = 0
for filename in os.listdir(test_dir):
    if filename.endswith('.rs'):
        filepath = os.path.join(test_dir, filename)
        if fix_test_file(filepath):
            fixed_count += 1

print(f"\nFixed {fixed_count} behavioral test files")
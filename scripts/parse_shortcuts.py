# This script is used to parse the markdown files in https://github.com/Fechin/reference/blob/main/source/_posts/
# And generate shortcuts mentioned in the markdown file.
# The generated json file can be imported by Liz.

import json
import sys
import os

def parse_markdown_to_json(md_file, json_file):
    data = []
    
    with open(md_file, 'r', encoding='utf-8') as f:
        content = f.read()
    
    current_section = None
    current_table = []
    tables_with_sections = []
    
    # First pass: Collect tables with their sections
    for line in content.split('\n'):
        stripped = line.strip()
        
        # Detect section headers (h3)
        if stripped.startswith('### '):
            # Extract section title and clean formatting
            section_title = stripped[4:].split(' {', 1)[0].strip()
            # Save previous table if any
            if current_table:
                tables_with_sections.append((current_section, current_table))
                current_table = []
            current_section = section_title
        elif stripped.startswith('|'):
            current_table.append(stripped)
        else:
            if current_table:
                tables_with_sections.append((current_section, current_table))
                current_table = []
    
    # Add any remaining table
    if current_table:
        tables_with_sections.append((current_section, current_table))
    
    # Process collected tables
    for section, table in tables_with_sections:
        if len(table) < 3:
            continue
            
        headers = [h.strip().lower() for h in table[0].split('|')[1:-1]]
        if headers != ['shortcut', 'action']:
            continue
        
        for row in table[2:]:
            columns = [col.strip() for col in row.split('|')[1:-1]]
            if len(columns) < 2:
                continue
                
            shortcut = columns[0].replace('`', '').strip()
            action = columns[1].strip()
            
            # Create combined description
            description = f"({section}) {action}" if section else action
            
            # Normalize shortcut format
            normalized_shortcut = shortcut.lower().replace(' ', '+')
            
            data.append({
                "description": description,
                "shortcut": normalized_shortcut,
                "application": "Blender",
                "comment": ""
            })
    
    with open(json_file, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=2, ensure_ascii=False)

# Usage
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python script.py <input.md> [output.json]")
        sys.exit(1)
    
    input_file = sys.argv[1]
    
    # Generate output filename
    if len(sys.argv) >= 3:
        output_file = sys.argv[2]
    else:
        # Replace .md with .json or append .json
        base_name = os.path.splitext(input_file)[0]
        output_file = f"{base_name}.json"
    
    parse_markdown_to_json(input_file, output_file)
    print(f"Converted {input_file} to {output_file}.")
    print(f"Warning: This action might not work all right, you should check {output_file} and make sure its content by yourself.")
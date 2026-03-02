#!/usr/bin/env bash

# --- Path-Aware Header ---
# Get the directory where this script is located (resolving symlinks)
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
# Assume the project root is one level up from the script's directory
PROJECT_ROOT=$( dirname "$SCRIPT_DIR" )

# Change to the project root. All other paths are relative to this.
cd "$PROJECT_ROOT" || { echo "FATAL: Could not change to project root"; exit 1; }
echo "Running in project root: $PWD"
# --- End Header ---

### An interactive script to tidy the project root.
### It asks before deleting or moving anything.

# --- Helper Function ---

# A simple Y/N prompt
_prompt_yes_no() {
    while true; do
        read -p "$1 (y/n) " answer
        case $answer in
            [Yy]* ) return 0;; # Success (true)
            [Nn]* ) return 1;; # Failure (false)
            * ) echo "Please answer 'y' or 'n'.";;
        esac
    done
}

# A function to handle duplicate files
_handle_duplicate() {
    local file1="$1"
    local file2="$2"
    echo ""
    echo "Found duplicate files:"
    echo "  [1] $file1"
    echo "  [2] $file2"
    
    while true; do
        read -p "Which version do you want to KEEP? (1 or 2, 's' to skip) " choice
        case $choice in
            1)
                if _prompt_yes_no "  -> Are you sure you want to delete '$file2'? "; then
                    rm -f "$file2"
                    echo "Removed $file2"
                fi
                break
                ;;
            2)
                if _prompt_yes_no "  -> Are you sure you want to delete '$file1'? "; then
                    rm -f "$file1"
                    echo "Removed $file1"
                fi
                break
                ;;
            [Ss]*)
                echo "Skipped."
                break
                ;;
            *)
                echo "Please enter 1, 2, or 's'."
                ;;
        esac
    done
}

# --- Main Script ---

echo "Starting interactive repository tidy-up..."
echo "Files will be moved to '_JUNK_TO_DELETE' (in the root) for safety."
echo "---"

# Create the junk directory
JUNK_DIR="_JUNK_TO_DELETE"
mkdir -p "$JUNK_DIR"

# ---
echo "Section 1: Build Artefacts & Misplaced Files"
# ---

ARTEFACTS=(
    '{{POLICY_JSON}}'
    '{{POLICY_JSON}}.prev'
    '{{POLICY_PRETTY}}'
    '{{TMP_DIR}}'
    'tree2.txt'
)

for item in "${ARTEFACTS[@]}"; do
    if [ -e "$item" ]; then
        if _prompt_yes_no "Move junk artefact '$item' to '$JUNK_DIR'? "; then
            mv "$item" "$JUNK_DIR"
            echo "Moved $item"
        fi
    fi
done

# Handle misplaced wordpress directory
if [ -d "wordpress" ] && [ -d "products/wordpress" ]; then
    if _prompt_yes_no "Move misplaced root 'wordpress/' dir to '$JUNK_DIR'? "; then
        mv "wordpress" "$JUNK_DIR"
        echo "Moved 'wordpress'"
    fi
fi

# ---
echo ""
echo "Section 2: node_modules"
# ---

# Add node_modules to .gitignore
GITIGNORE=".gitignore"
if [ -f "$GITIGNORE" ]; then
    if ! grep -q "node_modules" "$GITIGNORE"; then
        if _prompt_yes_no "Add 'node_modules/' to $GITIGNORE? "; then
            echo "" >> "$GITIGNORE"
            echo "# Node.js dependencies (build-time only)" >> "$GITIGNORE"
            echo "node_modules/" >> "$GITIGNORE"
            echo "Added 'node_modules/' to $GITIGNORE"
        fi
    fi
else
    if _prompt_yes_no "Create $GITIGNORE and add 'node_modules/'? "; then
        echo "node_modules/" > "$GITIGNORE"
        echo "Created $GITIGNORE"
    fi
fi

# Offer to delete node_modules
if [ -d "node_modules" ]; then
    if _prompt_yes_no "Delete local 'node_modules/' directory? (Saves space, run 'npm install' to rebuild) "; then
        rm -rf "node_modules"
        echo "Deleted 'node_modules/'"
    fi
fi

# ---
echo ""
echo "Section 3: File Duplicates"
# ---

if [ -f "bootstrap.sh" ] && [ -f "scripts/bootstrap.sh" ]; then
    _handle_duplicate "bootstrap.sh" "scripts/bootstrap.sh"
fi

if [ -f "crypto/sign_policy.js" ] && [ -f "policy/runner/crypto/sign_policy.js" ]; then
    _handle_duplicate "crypto/sign_policy.js" "policy/runner/crypto/sign_policy.js"
fi

if [ -f "policy/run.js" ] && [ -f "policy/runner/policy/run.js" ]; then
    _handle_duplicate "policy/run.js" "policy/runner/policy/run.js"
fi

# ---
echo ""
echo "Section 4: Documentation Duplicates"
# ---

DOC_DUPES=('default-consent.adoc' 'maintainer.adoc' 'trusted_contributor.adoc')
if [ -d "docs/capabilities" ]; then
    if _prompt_yes_no "Clean up duplicate docs? (This will keep the 'docs/capabilities/' versions) "; then
        for doc in "${DOC_DUPES[@]}"; do
            if [ -f "docs/$doc" ]; then
                rm -f "docs/$doc"
                echo "Removed docs/$doc"
            fi
        done
    fi
fi

# ---
echo ""
echo "Section 5: File Rename"
# ---

if [ -f "SECURITY.md" ]; then
    if _prompt_yes_no "Rename 'SECURITY.md' to 'SECURITY.adoc' (to match your preference)? "; then
        mv "SECURITY.md" "SECURITY.adoc"
        echo "Renamed to SECURITY.adoc"
    fi
fi

echo ""
echo "---"
echo "Interactive tidy-up complete."
echo "Please check the '$JUNK_DIR' directory and delete it when you are satisfied."

# Tree-sitter Queries for Nix

This directory contains tree-sitter query files that provide various editor features for the Nix language.

## Query Files

### highlights.scm
Syntax highlighting queries that define how different parts of Nix code should be colored:
- Keywords (`let`, `in`, `if`, `then`, `else`, `with`, `assert`, `rec`, `inherit`)
- Literals (numbers, strings, booleans, `null`)
- Operators (arithmetic, logical, comparison)
- Functions (built-in functions, function calls)
- Variables and parameters
- Comments
- String interpolation
- Paths and URIs

### folds.scm
Code folding regions for collapsing/expanding code blocks:
- Attribute sets and recursive attribute sets
- Lists
- Let expressions
- Function definitions
- If-then-else expressions
- Multi-line strings
- Comments

### indents.scm
Auto-indentation rules:
- Indent after opening brackets `{`, `[`, `(`
- Dedent on closing brackets `}`, `]`, `)`
- Special handling for `let...in` blocks
- Function body indentation
- Binary operation continuation
- Multi-line string handling

### locals.scm
Variable scoping and reference tracking:
- Scope definitions (functions, let expressions, attribute sets)
- Variable definitions and references
- Parameter definitions
- Field definitions in attribute sets
- Special handling for `rec` attribute sets
- Built-in variable recognition

### textobjects.scm
Text objects for vim-like motions and selections:
- Functions (inner/outer)
- Parameters
- Attribute sets
- Lists
- Let expressions
- Conditional expressions
- Strings
- Comments

### injections.scm
Language injection for embedded code:
- Bash scripts in derivation phases
- Python, Perl, Lua scripts
- JSON, TOML, YAML in parsing functions
- SQL queries
- HTML, CSS, JavaScript
- Configuration files
- Regular expressions in match functions

### tags.scm
Tag generation for jump-to-definition:
- Function definitions
- Package definitions
- Module definitions
- Service definitions
- Option definitions
- Flake outputs
- Overlay definitions

### context.scm
Context information for breadcrumbs and outline:
- Structural elements (let, function, attrset, list)
- Named sections (imports, options, config, services)
- Flake structure
- Package/derivation structure
- Hierarchical navigation

### matchup.scm
Bracket and pair matching:
- Brackets and braces `{}`, `[]`, `()`
- `let...in` pairs
- `if...then...else` chains
- String delimiters
- Interpolation delimiters

## Usage

### Neovim with nvim-treesitter

1. Install the Nix parser:
```vim
:TSInstall nix
```

2. The queries will be automatically used for:
   - Syntax highlighting
   - Code folding (with fold method set to `expr` or `syntax`)
   - Indentation
   - Text objects (with nvim-treesitter-textobjects)

### Helix

Copy the queries to your Helix runtime directory:
```bash
cp -r queries ~/.config/helix/runtime/queries/nix/
```

### Emacs with tree-sitter

The queries can be adapted for use with Emacs 29+ tree-sitter support.

### VSCode

These queries can be used with VSCode extensions that support tree-sitter.

## Supported Captures

### Highlighting Captures
- `@comment`
- `@string`
- `@string.escape`
- `@string.special.path`
- `@string.special.uri`
- `@number`
- `@number.float`
- `@constant.builtin`
- `@constant.default`
- `@keyword`
- `@keyword.operator`
- `@operator`
- `@punctuation.bracket`
- `@punctuation.delimiter`
- `@punctuation.special`
- `@function.call`
- `@function.builtin`
- `@variable`
- `@variable.builtin`
- `@variable.definition`
- `@parameter`
- `@parameter.definition`
- `@property`
- `@namespace`
- `@embedded`
- `@error`

### Fold Captures
- `@fold`

### Indent Captures
- `@indent`
- `@outdent`
- `@outdent.always`
- `@indent.always`
- `@indent.branch`
- `@indent.zero`
- `@align`
- `@auto`

### Local Captures
- `@scope`
- `@definition.var`
- `@definition.parameter`
- `@definition.field`
- `@reference`
- `@reference.field`
- `@reference.scope`

### Text Object Captures
- `@function.inner`/`@function.outer`
- `@parameter.inner`/`@parameter.outer`
- `@class.inner`/`@class.outer`
- `@block.inner`/`@block.outer`
- `@conditional.inner`/`@conditional.outer`
- `@string.inner`/`@string.outer`
- And many more...

## Customization

You can extend or override these queries by creating your own query files in your editor's configuration directory. Most editors support layering queries, allowing you to add custom highlights without modifying the base queries.

## Testing

Test the queries using tree-sitter CLI:
```bash
# Test highlighting
tree-sitter highlight test.nix

# Test query patterns
tree-sitter query queries/highlights.scm test.nix
```

## Contributing

When adding new queries:
1. Follow the existing naming conventions
2. Test with various Nix code samples
3. Consider edge cases and nested structures
4. Document any non-standard captures
5. Ensure compatibility with major editors
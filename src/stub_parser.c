// Stub parser for compilation
// This will be replaced by the actual Tree-sitter generated parser

void *tree_sitter_nix() {
    // Return a dummy pointer - this is just to make linking work
    // The actual parser will crash if used, but allows compilation
    static int dummy = 0;
    return &dummy;
}
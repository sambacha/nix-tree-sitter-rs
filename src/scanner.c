#include <tree_sitter/parser.h>
#include <string.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

enum TokenType {
    STRING_START,
    STRING_CONTENT,
    STRING_END,
    INDENTED_STRING_START,
    INDENTED_STRING_CONTENT,
    INDENTED_STRING_END,
    INTERPOLATION_START,
    INTERPOLATION_END,
    ESCAPE_SEQUENCE,
    COMMENT,
};

typedef struct {
    bool in_string;
    bool in_indented_string;
    int interpolation_depth;
    int paren_depth;
    int brace_depth;
} Scanner;

static inline void advance(TSLexer *lexer) {
    lexer->advance(lexer, false);
}

static inline void skip(TSLexer *lexer) {
    lexer->advance(lexer, true);
}

static bool scan_string_content(TSLexer *lexer) {
    bool has_content = false;
    lexer->mark_end(lexer);
    
    while (lexer->lookahead != 0) {
        if (lexer->lookahead == '"') {
            break;
        }
        if (lexer->lookahead == '\\') {
            break;
        }
        if (lexer->lookahead == '$') {
            advance(lexer);
            if (lexer->lookahead == '{') {
                // Back up - this is interpolation start
                return has_content;
            }
            has_content = true;
            lexer->mark_end(lexer);
        } else {
            advance(lexer);
            has_content = true;
            lexer->mark_end(lexer);
        }
    }
    return has_content;
}

static bool scan_indented_string_content(TSLexer *lexer) {
    bool has_content = false;
    int quote_count = 0;
    lexer->mark_end(lexer);
    
    while (lexer->lookahead != 0) {
        if (lexer->lookahead == '\'') {
            quote_count++;
            advance(lexer);
            if (quote_count == 2) {
                // Back up - might be end of string
                return has_content;
            }
            has_content = true;
            lexer->mark_end(lexer);
        } else {
            quote_count = 0;
            if (lexer->lookahead == '$') {
                advance(lexer);
                if (lexer->lookahead == '{') {
                    // Back up - this is interpolation start
                    return has_content;
                }
                has_content = true;
                lexer->mark_end(lexer);
            } else {
                advance(lexer);
                has_content = true;
                lexer->mark_end(lexer);
            }
        }
    }
    return has_content;
}

static bool scan_escape_sequence(TSLexer *lexer) {
    if (lexer->lookahead != '\\') return false;
    
    advance(lexer);
    
    switch (lexer->lookahead) {
        case 'n':
        case 'r':
        case 't':
        case '\\':
        case '"':
        case '\'':
        case '$':
            advance(lexer);
            return true;
        case 'x':
            advance(lexer);
            // Expect two hex digits
            for (int i = 0; i < 2; i++) {
                if ((lexer->lookahead >= '0' && lexer->lookahead <= '9') ||
                    (lexer->lookahead >= 'a' && lexer->lookahead <= 'f') ||
                    (lexer->lookahead >= 'A' && lexer->lookahead <= 'F')) {
                    advance(lexer);
                } else {
                    return false;
                }
            }
            return true;
        default:
            return false;
    }
}

static bool scan_comment(TSLexer *lexer) {
    if (lexer->lookahead == '#') {
        advance(lexer);
        while (lexer->lookahead != 0 && lexer->lookahead != '\n') {
            advance(lexer);
        }
        return true;
    }
    
    if (lexer->lookahead == '/') {
        advance(lexer);
        if (lexer->lookahead == '*') {
            advance(lexer);
            
            int depth = 1;
            while (depth > 0 && lexer->lookahead != 0) {
                if (lexer->lookahead == '/') {
                    advance(lexer);
                    if (lexer->lookahead == '*') {
                        advance(lexer);
                        depth++;
                    }
                } else if (lexer->lookahead == '*') {
                    advance(lexer);
                    if (lexer->lookahead == '/') {
                        advance(lexer);
                        depth--;
                    }
                } else {
                    advance(lexer);
                }
            }
            return depth == 0;
        }
    }
    
    return false;
}

void *tree_sitter_nix_external_scanner_create() {
    Scanner *scanner = calloc(1, sizeof(Scanner));
    return scanner;
}

void tree_sitter_nix_external_scanner_destroy(void *payload) {
    free(payload);
}

unsigned tree_sitter_nix_external_scanner_serialize(void *payload, char *buffer) {
    Scanner *scanner = (Scanner *)payload;
    buffer[0] = scanner->in_string;
    buffer[1] = scanner->in_indented_string;
    buffer[2] = scanner->interpolation_depth;
    buffer[3] = scanner->paren_depth;
    buffer[4] = scanner->brace_depth;
    return 5;
}

void tree_sitter_nix_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
    Scanner *scanner = (Scanner *)payload;
    if (length >= 5) {
        scanner->in_string = buffer[0];
        scanner->in_indented_string = buffer[1];
        scanner->interpolation_depth = buffer[2];
        scanner->paren_depth = buffer[3];
        scanner->brace_depth = buffer[4];
    }
}

bool tree_sitter_nix_external_scanner_scan(void *payload, TSLexer *lexer,
                                            const bool *valid_symbols) {
    Scanner *scanner = (Scanner *)payload;
    
    // Skip whitespace except when in strings
    if (!scanner->in_string && !scanner->in_indented_string) {
        while (lexer->lookahead == ' ' || lexer->lookahead == '\t' || 
               lexer->lookahead == '\n' || lexer->lookahead == '\r') {
            skip(lexer);
        }
    }
    
    // Handle comments
    if (valid_symbols[COMMENT] && scan_comment(lexer)) {
        lexer->result_symbol = COMMENT;
        return true;
    }
    
    // Handle string scanning
    if (!scanner->in_string && !scanner->in_indented_string) {
        // Check for string start
        if (valid_symbols[STRING_START] && lexer->lookahead == '"') {
            advance(lexer);
            scanner->in_string = true;
            lexer->result_symbol = STRING_START;
            return true;
        }
        
        // Check for indented string start
        if (valid_symbols[INDENTED_STRING_START] && lexer->lookahead == '\'') {
            advance(lexer);
            if (lexer->lookahead == '\'') {
                advance(lexer);
                scanner->in_indented_string = true;
                lexer->result_symbol = INDENTED_STRING_START;
                return true;
            }
        }
    }
    
    // Inside regular string
    if (scanner->in_string) {
        // Check for string end
        if (valid_symbols[STRING_END] && lexer->lookahead == '"') {
            advance(lexer);
            scanner->in_string = false;
            lexer->result_symbol = STRING_END;
            return true;
        }
        
        // Check for interpolation start
        if (valid_symbols[INTERPOLATION_START] && lexer->lookahead == '$') {
            advance(lexer);
            if (lexer->lookahead == '{') {
                advance(lexer);
                scanner->interpolation_depth++;
                scanner->brace_depth = 1;
                lexer->result_symbol = INTERPOLATION_START;
                return true;
            }
        }
        
        // Check for escape sequence
        if (valid_symbols[ESCAPE_SEQUENCE] && scan_escape_sequence(lexer)) {
            lexer->result_symbol = ESCAPE_SEQUENCE;
            return true;
        }
        
        // String content
        if (valid_symbols[STRING_CONTENT] && scan_string_content(lexer)) {
            lexer->result_symbol = STRING_CONTENT;
            return true;
        }
    }
    
    // Inside indented string
    if (scanner->in_indented_string) {
        // Check for indented string end
        if (valid_symbols[INDENTED_STRING_END] && lexer->lookahead == '\'') {
            advance(lexer);
            if (lexer->lookahead == '\'') {
                advance(lexer);
                scanner->in_indented_string = false;
                lexer->result_symbol = INDENTED_STRING_END;
                return true;
            }
        }
        
        // Check for interpolation start
        if (valid_symbols[INTERPOLATION_START] && lexer->lookahead == '$') {
            advance(lexer);
            if (lexer->lookahead == '{') {
                advance(lexer);
                scanner->interpolation_depth++;
                scanner->brace_depth = 1;
                lexer->result_symbol = INTERPOLATION_START;
                return true;
            }
        }
        
        // Indented string content
        if (valid_symbols[INDENTED_STRING_CONTENT] && scan_indented_string_content(lexer)) {
            lexer->result_symbol = INDENTED_STRING_CONTENT;
            return true;
        }
    }
    
    // Handle interpolation end
    if (scanner->interpolation_depth > 0 && valid_symbols[INTERPOLATION_END]) {
        // Track brace depth to find matching closing brace
        if (lexer->lookahead == '{') {
            scanner->brace_depth++;
            advance(lexer);
            return false;
        } else if (lexer->lookahead == '}') {
            scanner->brace_depth--;
            if (scanner->brace_depth == 0) {
                advance(lexer);
                scanner->interpolation_depth--;
                lexer->result_symbol = INTERPOLATION_END;
                return true;
            }
            advance(lexer);
            return false;
        }
    }
    
    return false;
}
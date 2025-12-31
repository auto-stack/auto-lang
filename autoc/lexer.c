/**
 * Lexer Implementation
 * Tokenizes auto-lang source code
 */

#include "autoc.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

// ============================================================================
// Lexer Creation
// ============================================================================

Lexer* lexer_new(const char* input) {
    Lexer* lexer = (Lexer*)malloc(sizeof(Lexer));
    lexer->input = input ? input : "";
    lexer->input_len = input ? strlen(input) : 0;
    lexer->pos = 0;
    lexer->line = 1;
    lexer->at = 0;
    lexer->fstr_note = '$';
    lexer->buffer = NULL;
    lexer->buffer_count = 0;
    lexer->buffer_capacity = 0;
    lexer->last.kind = TOKEN_EOF;
    lexer->last.text = astr_new("");
    lexer->last.pos.line = 0;
    lexer->last.pos.at = 0;
    lexer->last.pos.pos = 0;
    lexer->last.pos.len = 0;
    return lexer;
}

void lexer_free(Lexer* lexer) {
    if (!lexer) return;
    if (lexer->buffer) {
        for (size_t i = 0; i < lexer->buffer_count; i++) {
            astr_free(&lexer->buffer[i].text);
        }
        free(lexer->buffer);
    }
    // Don't free lexer->last.text - its ownership was transferred to parser
    free(lexer);
}

void lexer_set_fstr_note(Lexer* lexer, char note) {
    lexer->fstr_note = note;
}

// ============================================================================
// Lexer Utilities
// ============================================================================

static char lexer_peek(Lexer* lexer, size_t offset) {
    if (lexer->pos + offset < lexer->input_len) {
        return lexer->input[lexer->pos + offset];
    }
    return '\0';
}

static char lexer_current(Lexer* lexer) {
    return lexer_peek(lexer, 0);
}

static void lexer_advance(Lexer* lexer) {
    if (lexer->pos < lexer->input_len) {
        lexer->pos++;
        lexer->at++;
    }
}

static void lexer_skip_whitespace(Lexer* lexer) {
    while (lexer->pos < lexer->input_len) {
        char c = lexer_current(lexer);
        if (c == ' ' || c == '\t' || c == '\r') {
            lexer_advance(lexer);
        } else {
            break;
        }
    }
}

static Pos lexer_pos(Lexer* lexer, size_t len) {
    Pos p;
    p.line = lexer->line;
    p.at = lexer->at;
    p.pos = lexer->pos;
    p.len = len;
    return p;
}

static Token lexer_make_token(Lexer* lexer, TokenKind kind, const char* text, size_t len) {
    Token token;
    token.kind = kind;
    token.pos = lexer_pos(lexer, len);
    token.text = astr_from_len(text, len);
    return token;
}

// ============================================================================
// Number Parsing
// ============================================================================

static Token lexer_number(Lexer* lexer) {
    size_t start = lexer->pos;
    bool has_dot = false;
    bool is_hex = false;

    // Check for hex prefix
    if (lexer_current(lexer) == '0' && lexer_peek(lexer, 1) == 'x') {
        is_hex = true;
        lexer_advance(lexer); // skip 0
        lexer_advance(lexer); // skip x
    }

    while (lexer->pos < lexer->input_len) {
        char c = lexer_current(lexer);
        if (c == '_') {
            lexer_advance(lexer);
            continue;
        }
        if (c == '.' && !has_dot) {
            // Check if this is a valid float (has digit after)
            if (isdigit((unsigned char)lexer_peek(lexer, 1))) {
                has_dot = true;
                lexer_advance(lexer);
                continue;
            }
            break;
        }
        if (is_hex ? isxdigit((unsigned char)c) : isdigit((unsigned char)c)) {
            lexer_advance(lexer);
            continue;
        }
        break;
    }

    // Check for type suffix
    TokenKind kind = TOKEN_INT;
    if (lexer->pos < lexer->input_len) {
        char c = lexer_current(lexer);
        if (c == 'f') {
            kind = TOKEN_FLOAT;
            lexer_advance(lexer);
        } else if (c == 'd') {
            kind = TOKEN_DOUBLE;
            lexer_advance(lexer);
        } else if (c == 'u') {
            kind = TOKEN_UINT;
            lexer_advance(lexer);
            if (lexer_current(lexer) == '8') {
                kind = TOKEN_U8;
                lexer_advance(lexer);
            }
        } else if (c == 'i') {
            kind = TOKEN_INT;
            lexer_advance(lexer);
            if (lexer_current(lexer) == '8') {
                kind = TOKEN_I8;
                lexer_advance(lexer);
            }
        } else if (has_dot || kind == TOKEN_FLOAT || kind == TOKEN_DOUBLE) {
            kind = TOKEN_FLOAT;
        }
    }

    if (has_dot) kind = TOKEN_FLOAT;

    return lexer_make_token(lexer, kind, lexer->input + start, lexer->pos - start);
}

// ============================================================================
// String and Character Parsing
// ============================================================================

static Token lexer_string(Lexer* lexer) {
    lexer_advance(lexer); // skip opening quote
    size_t start = lexer->pos;

    while (lexer->pos < lexer->input_len) {
        char c = lexer_current(lexer);
        if (c == '"') {
            Token token = lexer_make_token(lexer, TOKEN_STR, lexer->input + start, lexer->pos - start);
            lexer_advance(lexer); // skip closing quote
            return token;
        }
        if (c == '\\' && lexer_peek(lexer, 1)) {
            lexer_advance(lexer); // skip backslash
            lexer_advance(lexer); // skip escaped char
            continue;
        }
        lexer_advance(lexer);
    }

    // Unclosed string - return what we have
    return lexer_make_token(lexer, TOKEN_STR, lexer->input + start, lexer->pos - start);
}

static Token lexer_cstring(Lexer* lexer) {
    lexer_advance(lexer); // skip c
    lexer_advance(lexer); // skip opening quote
    size_t start = lexer->pos;

    while (lexer->pos < lexer->input_len) {
        char c = lexer_current(lexer);
        if (c == '"') {
            Token token = lexer_make_token(lexer, TOKEN_CSTR, lexer->input + start, lexer->pos - start);
            lexer_advance(lexer); // skip closing quote
            return token;
        }
        lexer_advance(lexer);
    }

    return lexer_make_token(lexer, TOKEN_CSTR, lexer->input + start, lexer->pos - start);
}

static Token lexer_char(Lexer* lexer) {
    lexer_advance(lexer); // skip opening quote
    size_t start = lexer->pos;

    if (lexer->pos < lexer->input_len) {
        char c = lexer_current(lexer);
        lexer_advance(lexer);

        // Handle escape sequences
        if (c == '\\' && lexer->pos < lexer->input_len) {
            lexer_advance(lexer);
        }

        // Skip closing quote
        if (lexer->pos < lexer->input_len && lexer_current(lexer) == '\'') {
            lexer_advance(lexer);
        }

        return lexer_make_token(lexer, TOKEN_CHAR, lexer->input + start, 1);
    }

    return lexer_make_token(lexer, TOKEN_CHAR, lexer->input + start, 1);
}

// ============================================================================
// Format String Parsing
// ============================================================================

static void lexer_buffer_push(Lexer* lexer, Token token) {
    if (lexer->buffer_count >= lexer->buffer_capacity) {
        lexer->buffer_capacity = lexer->buffer_capacity == 0 ? 16 : lexer->buffer_capacity * 2;
        lexer->buffer = (Token*)realloc(lexer->buffer, lexer->buffer_capacity * sizeof(Token));
    }
    lexer->buffer[lexer->buffer_count++] = token;
}

static Token lexer_fstr(Lexer* lexer) {
    char end_char = '`';
    bool is_tick = lexer_current(lexer) == '`';

    if (!is_tick) {
        end_char = '"';
        lexer_advance(lexer); // skip f
    }

    lexer_advance(lexer); // skip opening quote/backtick

    // Emit FSTR_START
    Pos start_pos = lexer_pos(lexer, 0);
    Token start_token;
    start_token.kind = TOKEN_FSTR_START;
    start_token.pos = start_pos;
    start_token.text = astr_new(is_tick ? "`" : "f\"");
    lexer_buffer_push(lexer, start_token);

    AutoStr text = astr_new("");
    while (lexer->pos < lexer->input_len) {
        char c = lexer_current(lexer);

        if (c == end_char) {
            // Emit pending text part
            if (text.len > 0) {
                Token part;
                part.kind = TOKEN_FSTR_PART;
                part.pos = lexer_pos(lexer, text.len);
                part.text = text;
                lexer_buffer_push(lexer, part);
            }
            lexer_advance(lexer); // skip closing quote

            // Emit FSTR_END
            Token end_token;
            end_token.kind = TOKEN_FSTR_END;
            end_token.pos = lexer_pos(lexer, 0);
            end_token.text = astr_new(is_tick ? "`" : "\"");
            lexer_buffer_push(lexer, end_token);

            // Return the first buffered token
            Token first = lexer->buffer[0];
            // Shift buffer
            for (size_t i = 1; i < lexer->buffer_count; i++) {
                lexer->buffer[i - 1] = lexer->buffer[i];
            }
            lexer->buffer_count--;
            return first;
        }

        if (c == lexer->fstr_note) {
            // Emit text part before $
            if (text.len > 0) {
                Token part;
                part.kind = TOKEN_FSTR_PART;
                part.pos = lexer_pos(lexer, text.len);
                part.text = text;
                lexer_buffer_push(lexer, part);
                text = astr_new("");
            }

            // Emit FSTR_NOTE ($)
            Token note;
            note.kind = TOKEN_FSTR_NOTE;
            note.pos = lexer_pos(lexer, 1);
            note.text = astr_from_len(&c, 1);
            lexer_buffer_push(lexer, note);
            lexer_advance(lexer);

            // Check for ${expr}
            if (lexer_current(lexer) == '{') {
                // Push tokens until matching }
                int depth = 1;
                lexer_buffer_push(lexer, lexer_make_token(lexer, TOKEN_LBRACE, "{", 1));
                lexer_advance(lexer);

                while (lexer->pos < lexer->input_len && depth > 0) {
                    char bc = lexer_current(lexer);
                    if (bc == '{') depth++;
                    if (bc == '}') depth--;

                    Token t = lexer_next(lexer);
                    lexer_buffer_push(lexer, t);

                    if (depth == 0) break;
                }
            } else {
                // Simple identifier
                size_t ident_start = lexer->pos;
                while (lexer->pos < lexer->input_len) {
                    char ic = lexer_current(lexer);
                    if (isalnum((unsigned char)ic) || ic == '_') {
                        lexer_advance(lexer);
                    } else {
                        break;
                    }
                }
                Token ident = lexer_make_token(lexer, TOKEN_IDENT, lexer->input + ident_start, lexer->pos - ident_start);
                lexer_buffer_push(lexer, ident);
            }
        } else {
            astr_append_char(&text, c);
            lexer_advance(lexer);
        }
    }

    // Unclosed fstr - emit what we have
    if (text.len > 0) {
        Token part;
        part.kind = TOKEN_FSTR_PART;
        part.pos = lexer_pos(lexer, text.len);
        part.text = text;
        lexer_buffer_push(lexer, part);
    }

    Token first = lexer->buffer[0];
    // Shift buffer
    for (size_t i = 1; i < lexer->buffer_count; i++) {
        lexer->buffer[i - 1] = lexer->buffer[i];
    }
    lexer->buffer_count--;
    return first;
}

// ============================================================================
// Identifier and Keyword Parsing
// ============================================================================

static Token lexer_identifier(Lexer* lexer) {
    size_t start = lexer->pos;

    // First char must be letter or underscore
    if (lexer->pos < lexer->input_len) {
        char c = lexer_current(lexer);
        if (!isalpha((unsigned char)c) && c != '_') {
            return lexer_make_token(lexer, TOKEN_EOF, "", 0);
        }
        lexer_advance(lexer);
    }

    // Rest can be alphanumeric or underscore
    while (lexer->pos < lexer->input_len) {
        char c = lexer_current(lexer);
        if (isalnum((unsigned char)c) || c == '_') {
            lexer_advance(lexer);
        } else {
            break;
        }
    }

    size_t len = lexer->pos - start;
    const char* text = lexer->input + start;

    // Check for keywords
    #define CHECK_KEYWORD(s, k) \
        if (len == strlen(s) && memcmp(text, s, len) == 0) { \
            return lexer_make_token(lexer, k, text, len); \
        }

    CHECK_KEYWORD("true", TOKEN_TRUE);
    CHECK_KEYWORD("false", TOKEN_FALSE);
    CHECK_KEYWORD("nil", TOKEN_NIL);
    CHECK_KEYWORD("null", TOKEN_NULL);
    CHECK_KEYWORD("if", TOKEN_IF);
    CHECK_KEYWORD("else", TOKEN_ELSE);
    CHECK_KEYWORD("for", TOKEN_FOR);
    CHECK_KEYWORD("when", TOKEN_WHEN);
    CHECK_KEYWORD("is", TOKEN_IS);
    CHECK_KEYWORD("var", TOKEN_VAR);
    CHECK_KEYWORD("in", TOKEN_IN);
    CHECK_KEYWORD("fn", TOKEN_FN);
    CHECK_KEYWORD("type", TOKEN_TYPE);
    CHECK_KEYWORD("union", TOKEN_UNION);
    CHECK_KEYWORD("tag", TOKEN_TAG);
    CHECK_KEYWORD("let", TOKEN_LET);
    CHECK_KEYWORD("mut", TOKEN_MUT);
    CHECK_KEYWORD("has", TOKEN_HAS);
    CHECK_KEYWORD("use", TOKEN_USE);
    CHECK_KEYWORD("as", TOKEN_AS);
    CHECK_KEYWORD("enum", TOKEN_ENUM);
    CHECK_KEYWORD("on", TOKEN_ON);
    CHECK_KEYWORD("alias", TOKEN_ALIAS);
    CHECK_KEYWORD("break", TOKEN_BREAK);
    CHECK_KEYWORD("grid", TOKEN_GRID);

    #undef CHECK_KEYWORD

    return lexer_make_token(lexer, TOKEN_IDENT, text, len);
}

// ============================================================================
// Comment Parsing
// ============================================================================

static Token lexer_comment(Lexer* lexer) {
    lexer_advance(lexer); // skip first /

    if (lexer_current(lexer) == '/') {
        // Line comment
        lexer_advance(lexer); // skip second /
        size_t start = lexer->pos;

        while (lexer->pos < lexer->input_len && lexer_current(lexer) != '\n') {
            lexer_advance(lexer);
        }

        Token content = lexer_make_token(lexer, TOKEN_COMMENT_CONTENT,
                                         lexer->input + start, lexer->pos - start);
        // Buffer the content
        lexer_buffer_push(lexer, content);

        // Return the line comment token
        Token token;
        token.kind = TOKEN_COMMENT_LINE;
        token.pos = lexer_pos(lexer, 2);
        token.text = astr_new("//");
        return token;
    }

    if (lexer_current(lexer) == '*') {
        // Block comment
        lexer_advance(lexer); // skip *
        size_t start = lexer->pos;

        while (lexer->pos < lexer->input_len) {
            if (lexer_current(lexer) == '*' && lexer_peek(lexer, 1) == '/') {
                Token content = lexer_make_token(lexer, TOKEN_COMMENT_CONTENT,
                                                 lexer->input + start, lexer->pos - start);
                lexer_buffer_push(lexer, content);

                Token start_token;
                start_token.kind = TOKEN_COMMENT_START;
                start_token.pos = lexer_pos(lexer, 2);
                start_token.text = astr_new("/*");
                lexer_buffer_push(lexer, start_token);

                lexer_advance(lexer); // skip *
                lexer_advance(lexer); // skip /

                Token end_token;
                end_token.kind = TOKEN_COMMENT_END;
                end_token.pos = lexer_pos(lexer, 0);
                end_token.text = astr_new("*/");
                lexer_buffer_push(lexer, end_token);

                Token first = lexer->buffer[0];
                // Shift buffer
                for (size_t i = 1; i < lexer->buffer_count; i++) {
                    lexer->buffer[i - 1] = lexer->buffer[i];
                }
                lexer->buffer_count--;
                return first;
            }
            lexer_advance(lexer);
        }

        // Unclosed comment - return EOF
        Token token;
        token.kind = TOKEN_EOF;
        token.pos = lexer_pos(lexer, 0);
        token.text = astr_new("");
        return token;
    }

    // Just a division operator
    return lexer_make_token(lexer, TOKEN_DIV, "/", 1);
}

// ============================================================================
// Main Tokenization
// ============================================================================

Token lexer_next(Lexer* lexer) {
    // Check buffer first
    if (lexer->buffer_count > 0) {
        Token token = lexer->buffer[0];
        // Shift buffer
        for (size_t i = 1; i < lexer->buffer_count; i++) {
            lexer->buffer[i - 1] = lexer->buffer[i];
        }
        lexer->buffer_count--;
        lexer->last = token;
        return token;
    }

    lexer_skip_whitespace(lexer);

    if (lexer->pos >= lexer->input_len) {
        Token token;
        token.kind = TOKEN_EOF;
        token.pos = lexer_pos(lexer, 0);
        token.text = astr_new("");
        lexer->last = token;
        return token;
    }

    char c = lexer_current(lexer);

    // Newline
    if (c == '\n') {
        lexer->line++;
        size_t old_at = lexer->at;
        lexer->at = 0;
        lexer_advance(lexer);
        Token token = lexer_make_token(lexer, TOKEN_NEWLINE, "\n", 1);
        lexer->last = token;
        return token;
    }

    // Numbers
    if (isdigit((unsigned char)c)) {
        Token token = lexer_number(lexer);
        lexer->last = token;
        return token;
    }

    // Strings and chars
    if (c == '"') {
        // Check for c"..." string
        if (lexer_peek(lexer, -1) == 'c' && lexer->pos > 0 && lexer->input[lexer->pos - 1] == 'c') {
            // Adjust for the already consumed 'c'
            lexer->pos--;
            Token token = lexer_cstring(lexer);
            lexer->last = token;
            return token;
        }
        // Check for f"..." format string
        if (lexer_peek(lexer, -1) == 'f' && lexer->pos > 0 && lexer->input[lexer->pos - 1] == 'f') {
            lexer->pos--;
            Token token = lexer_fstr(lexer);
            lexer->last = token;
            return token;
        }
        Token token = lexer_string(lexer);
        lexer->last = token;
        return token;
    }

    if (c == '`') {
        Token token = lexer_fstr(lexer);
        lexer->last = token;
        return token;
    }

    if (c == '\'') {
        Token token = lexer_char(lexer);
        lexer->last = token;
        return token;
    }

    // Comments and division
    if (c == '/') {
        Token token = lexer_comment(lexer);
        lexer->last = token;
        return token;
    }

    // Operators and punctuation
    switch (c) {
        case '(':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_LPAREN, "(", 1);
        case ')':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_RPAREN, ")", 1);
        case '[':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_LSQUARE, "[", 1);
        case ']':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_RSQUARE, "]", 1);
        case '{':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_LBRACE, "{", 1);
        case '}':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_RBRACE, "}", 1);
        case ',':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_COMMA, ",", 1);
        case ';':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_SEMI, ";", 1);
        case ':':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_COLON, ":", 1);
        case '|':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_VBAR, "|", 1);
        case '?':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_QUESTION, "?", 1);
        case '@':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_AT, "@", 1);
        case '#':
            lexer_advance(lexer);
            return lexer_make_token(lexer, TOKEN_HASH, "#", 1);
        case '+':
            lexer_advance(lexer);
            if (lexer_current(lexer) == '=') {
                lexer_advance(lexer);
                return lexer_make_token(lexer, TOKEN_ADDEQ, "+=", 2);
            }
            return lexer_make_token(lexer, TOKEN_ADD, "+", 1);
        case '-':
            lexer_advance(lexer);
            if (lexer_current(lexer) == '>') {
                lexer_advance(lexer);
                return lexer_make_token(lexer, TOKEN_ARROW, "->", 2);
            }
            if (lexer_current(lexer) == '=') {
                lexer_advance(lexer);
                return lexer_make_token(lexer, TOKEN_SUBEQ, "-=", 2);
            }
            return lexer_make_token(lexer, TOKEN_SUB, "-", 1);
        case '*':
            lexer_advance(lexer);
            if (lexer_current(lexer) == '=') {
                lexer_advance(lexer);
                return lexer_make_token(lexer, TOKEN_MULEQ, "*=", 2);
            }
            return lexer_make_token(lexer, TOKEN_STAR, "*", 1);
        case '=':
            lexer_advance(lexer);
            if (lexer_current(lexer) == '=') {
                lexer_advance(lexer);
                return lexer_make_token(lexer, TOKEN_EQ, "==", 2);
            }
            if (lexer_current(lexer) == '>') {
                lexer_advance(lexer);
                return lexer_make_token(lexer, TOKEN_DOUBLE_ARROW, "=>", 2);
            }
            return lexer_make_token(lexer, TOKEN_ASN, "=", 1);
        case '!':
            lexer_advance(lexer);
            if (lexer_current(lexer) == '=') {
                lexer_advance(lexer);
                return lexer_make_token(lexer, TOKEN_NEQ, "!=", 2);
            }
            return lexer_make_token(lexer, TOKEN_NOT, "!", 1);
        case '<':
            lexer_advance(lexer);
            if (lexer_current(lexer) == '=') {
                lexer_advance(lexer);
                return lexer_make_token(lexer, TOKEN_LE, "<=", 2);
            }
            return lexer_make_token(lexer, TOKEN_LT, "<", 1);
        case '>':
            lexer_advance(lexer);
            if (lexer_current(lexer) == '=') {
                lexer_advance(lexer);
                return lexer_make_token(lexer, TOKEN_GE, ">=", 2);
            }
            return lexer_make_token(lexer, TOKEN_GT, ">", 1);
        case '.':
            lexer_advance(lexer);
            if (lexer_current(lexer) == '.') {
                lexer_advance(lexer);
                if (lexer_current(lexer) == '=') {
                    lexer_advance(lexer);
                    return lexer_make_token(lexer, TOKEN_RANGEEQ, "..=", 3);
                }
                return lexer_make_token(lexer, TOKEN_RANGE, "..", 2);
            }
            return lexer_make_token(lexer, TOKEN_DOT, ".", 1);
    }

    // Identifier or keyword
    if (isalpha((unsigned char)c) || c == '_') {
        Token token = lexer_identifier(lexer);
        lexer->last = token;
        return token;
    }

    // Unknown character
    lexer_advance(lexer);
    return lexer_make_token(lexer, TOKEN_EOF, &c, 1);
}

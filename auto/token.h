#pragma once

enum TokenKind {
    TOKENKIND_I8Lit = 0,
    TOKENKIND_U8Lit = 1,
    TOKENKIND_I16Lit = 2,
    TOKENKIND_U16Lit = 3,
    TOKENKIND_I32Lit = 4,
    TOKENKIND_U32Lit = 5,
    TOKENKIND_I64Lit = 6,
    TOKENKIND_U64Lit = 7,
    TOKENKIND_DecLit = 8,
    TOKENKIND_FloatLit = 9,
    TOKENKIND_DoubleLit = 10,
    TOKENKIND_StrLit = 11,
    TOKENKIND_CStrLit = 12,
    TOKENKIND_CharLit = 13,
    TOKENKIND_RuneLit = 14,
    TOKENKIND_LParen = 15,
    TOKENKIND_RParen = 16,
    TOKENKIND_LSquare = 17,
    TOKENKIND_RSquare = 18,
    TOKENKIND_LBrace = 19,
    TOKENKIND_RBrace = 20,
    TOKENKIND_Let = 21,
    TOKENKIND_Var = 22,
    TOKENKIND_Const = 23,
    TOKENKIND_Alias = 24,
    TOKENKIND_Type = 25,
    TOKENKIND_In = 26,
    TOKENKIND_Mut = 27,
    TOKENKIND_Out = 28,
};
struct Pos {
    unsigned int line;
    unsigned int lpos;
    unsigned int spos;
};
struct Token {
    enum TokenKind kind;
    struct Pos pos;
    char* text;
};

#pragma once

struct Pos {
    unsigned int line;
    unsigned int pos;
};
enum TokenKind {
    TOKENKIND_Int = 0,
    TOKENKIND_UInt = 1,
    TOKENKIND_I8 = 2,
    TOKENKIND_U8 = 3,
    TOKENKIND_Float = 4,
    TOKENKIND_Double = 5,
    TOKENKIND_Str = 6,
    TOKENKIND_CStr = 7,
    TOKENKIND_Char = 8,
    TOKENKIND_Rune = 9,
    TOKENKIND_LParen = 10,
    TOKENKIND_RParen = 11,
    TOKENKIND_LSquare = 12,
    TOKENKIND_RSquare = 13,
    TOKENKIND_LBrace = 14,
    TOKENKIND_RBrace = 15,
};

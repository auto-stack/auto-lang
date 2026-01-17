#pragma once

enum OptionKind {
    OPTION_SOME,
    OPTION_NONE,
};

struct Option {
    enum OptionKind tag;
    union {
        int Some;
        int None;
    } as;
};
int double_or_default(struct Option opt, int default);
int get_value(struct Option opt);

struct Version {
    int major;
    int minor;
    int patch;
};

struct SurnRegistry {
    char *lang_name;
    char *lang_desc;
    struct Version lang_version;
    int api_version;
};

struct SurnAST {
    char *name;
    char *surname;
    struct Version version;
};

enum AstNode {
    Statement = 1,
    Expression = 2,
    Macro = 3,
};

extern void register_surn(struct SurnRegistry* reg);
extern char* transform(struct SurnAST* ast);
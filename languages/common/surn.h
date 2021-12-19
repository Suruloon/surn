struct SurnRegistry {
    char *lang_name;
    char *lang_desc;
    version lang_version;
    int api_version;
};

struct Version {
    int major;
    int minor;
    int patch;
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

extern SurnRegistry register();
extern void transform(SurnAST *ast);
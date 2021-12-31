// this is the API for PHP 8.0.0
#include "../common/surn.h"


extern void register_surn(struct SurnRegistry* reg) {
    char lang_name[3] = "php";
    char lang_desc[22] = "Support for PHP 8.x.x";
    int api_version = 1;
    struct Version lang_version = {8, 0, 0};
    reg->lang_name = lang_name;
    reg->lang_desc = lang_desc;
    reg->lang_version = lang_version;
    reg->api_version = api_version;
    return reg;
};

extern char* transform(struct SurnAST* ast) {
    // transform nodes here...
    char php_body[] = "<php\n";

    push(&php_body, "echo \"Hello, World!\";");
};

void push(char* string, char* to_write) {
    int size = strlen(string);
    for (int i = 0; string[size] != '\0'; i++) {
        string[i] = to_write;
        string[i + 1] = '\0';
    }
}

int strlen(char* string) {
    int size = 0;
    for (char* c = string; *c; c++) {
        size++;
    }
    return size;
}
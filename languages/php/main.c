// this is the API for PHP 8.0.0
#include "../common/surn.h"


SurnAST register() {
    char lang_name[] = "php-8.0.0";
    char lang_desc[] = "Support for PHP 8.x.x";
    int api_version = 1;
    version lang_version = {8, 0, 0};
    SurnRegistry reg = {lang_name, lang_desc, lang_version, api_version};
    return reg;
};

void transform(SurnAST *ast) {
    
};
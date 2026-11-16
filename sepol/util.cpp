//
// Created by chic on 2025/10/29.
//
#include "util.h"
#include <string>
using namespace std;

sFILE make_file(FILE *fp) {
    return sFILE(fp, [](FILE *fp){ return fp ? fclose(fp) : 1; });
}

void file_readline(bool trim, FILE *fp, const function<bool(string_view)> &fn) {
    size_t len = 1024;
    char *buf = (char *) malloc(len);
    char *start;
    ssize_t read;
    while ((read = getline(&buf, &len, fp)) >= 0) {
        start = buf;
        if (trim) {
            while (read && "\n\r "sv.find(buf[read - 1]) != string::npos)
                --read;
            buf[read] = '\0';
            while (*start == ' ')
                ++start;
        }
        if (!fn(start))
            break;
    }
    fclose(fp);
    free(buf);
}
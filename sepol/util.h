//
// Created by chic on 2025/10/29.
//

#pragma once

#include <memory>
#include <cstdio>
#include <string_view>
#include <functional>

using sFILE = std::unique_ptr<FILE, decltype(&fclose)>;

sFILE make_file(FILE *fp) ;
void file_readline(bool trim, FILE *fp, const std::function<bool(std::string_view)> &fn);

static inline void file_readline(
        bool trim, const char *file, const std::function<bool(std::string_view)> &fn) {
    FILE *fp = fopen(file, "re");
    if (fp == nullptr)
        return;
    file_readline(trim, fp, fn);
}

static inline void file_readline(const char *file,
                                 const std::function<bool(std::string_view)> &fn) {
    file_readline(false, file, fn);
}


static inline sFILE open_file(const char *path, const char *mode) {
    return make_file(fopen(path, mode));
}



#define DISALLOW_COPY_AND_MOVE(clazz) \
clazz(const clazz &) = delete; \
clazz(clazz &&) = delete;


template <class Func>
class run_finally {
    DISALLOW_COPY_AND_MOVE(run_finally)
public:
    explicit run_finally(Func &&fn) : fn(std::move(fn)) {}
    ~run_finally() { fn(); }
private:
    Func fn;
};
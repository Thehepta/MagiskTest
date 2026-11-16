//
// Created by chic on 2025/11/3.
//

#pragma once

#include "sepolicy.h"
#include <memory>
#include "export.h"

extern "C" int entry(char *data, size_t len) {

    auto sepol = std::unique_ptr<sepolicy>(sepolicy::from_data(data,len));
    sepol->magisk_rules();
//    sepol->load_rules(selinux_rule_file);
    // Load patched policy into kernel
    sepol->to_file("/sys/fs/selinux/load");
    return 0;
}

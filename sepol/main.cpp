#include <iostream>
#include "sepolicy.h"
#include <memory>

int main(int argc,char**argv) {

    char * selinux_input_data_file = argv[1];
    printf("selinux_input_data_file:%s\n",selinux_input_data_file);
//    char * selinux_rule_file = argv[2];
    char * selinux_output_data_file = argv[2];
    printf("selinux_output_data_file:%s\n",selinux_output_data_file);

    auto sepol = std::unique_ptr<sepolicy>(sepolicy::from_file(selinux_input_data_file));
    sepol->magisk_rules();
//    sepol->load_rules(selinux_rule_file);

    // Load patched policy into kernel
    sepol->to_file(selinux_output_data_file);
    return 0;
}

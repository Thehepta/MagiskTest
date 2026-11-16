#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>
#include <dlfcn.h>
#include "fcntl.h"
#include <unistd.h>

__attribute__((constructor))
static void preload_init() {

    int fd = open("/dev/kmsg", O_WRONLY | O_CLOEXEC);
    if (fd >= 0) {
        const char *msg = "<6>magisk debug log\n";
        write(fd, msg, 20);
        close(fd);
    }

    // Make sure our next exec won't get bugged
    unsetenv("LD_PRELOAD");
}

int security_load_policy(void *data, size_t len) {

    int fd = open("/dev/kmsg", O_WRONLY | O_CLOEXEC);
    if (fd >= 0) {
        const char *msg = "<6>magisk: security_load_policy\n";
        write(fd, msg, 32);
        close(fd);
    }
    void * handle = dlopen("/dev/libsepolicy.so",RTLD_GLOBAL);
    int (* entry_fun)(void *, size_t) = (int (*)(void *data, size_t len))dlsym(handle,"entry");
    entry_fun(data,len);
    return 0;
}




// 声明原始 malloc 函数指针
//static int* (*orig_security_load_policy)(void *data, size_t len) = NULL;
//
//int security_load_policy(void *data, size_t len) {
//    // 第一次调用时解析原始 malloc
//    if (!orig_security_load_policy) {
//        orig_security_load_policy = (int* (*)(void *data, size_t len)) dlsym(RTLD_NEXT, "malloc");
//        if (!orig_security_load_policy) {
//            // 处理错误（注意：不能调用 malloc，否则死循环）
//            write(STDERR_FILENO, "dlsym failed\n", 13);
//            _exit(1);
//        }
//    }
//
//
//    // 调用原始 malloc
//    void* ptr = orig_security_load_policy(data,len);
//
//    // 可选：后处理
//    // fprintf(stderr, "[HOOK] malloc returned %p\n", ptr);
//
//    return 0;
//}
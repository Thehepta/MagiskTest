#pragma once


#ifndef LOG_TAG
#if defined(__LP64__)
# define LOG_TAG "sepol"
#else
# define LOG_TAG "sepol32"
#endif
#endif

#define NDEBUG



#if defined(__ANDROID__)
// Android 编译环境

#include <android/log.h>

#ifndef NDEBUG
#define LOGD(...)  logging::log(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#define LOGV(...)  logging::log(ANDROID_LOG_VERBOSE, LOG_TAG, __VA_ARGS__)
#else
#define LOGD(...)
#define LOGV(...)
#endif

#define LOGI(...)  logging::log(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGW(...)  logging::log(ANDROID_LOG_WARN, LOG_TAG, __VA_ARGS__)
#define LOGE(...)  logging::log(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
#define LOGF(...)  logging::log(ANDROID_LOG_FATAL, LOG_TAG, __VA_ARGS__)
#define PLOGE(fmt, args...) LOGE("[-][fun:%s]:" fmt " %d: %s", __func__,##args, errno, strerror(errno))
#define DEBUG(fmt, args...) LOGD("[+][fun:%s]:" fmt, __func__,##args)


#else

#include <errno.h>
#include <string.h>


#define LOGI(...)  logging::log(-1, LOG_TAG, __VA_ARGS__)
#define LOGD(...)  logging::log(-1, LOG_TAG, __VA_ARGS__)
#define LOGW(...)  logging::log(-1, LOG_TAG, __VA_ARGS__)
#define LOGE(...)  logging::log(-1, LOG_TAG, __VA_ARGS__)
#define LOGF(...)  logging::log(-1, LOG_TAG, __VA_ARGS__)
#define PLOGE(fmt, args...) LOGE("[-][fun:%s]:" fmt " %d: %s", __func__,##args, errno, strerror(errno))
#define DEBUG(fmt, args...) LOGD("[+][fun:%s]:" fmt, __func__,##args)


#endif

namespace logging {
    void setfd(int fd);

    int getfd();

    [[gnu::format(printf, 3, 4)]]
    void log(int prio, const char* tag, const char* fmt, ...);
}
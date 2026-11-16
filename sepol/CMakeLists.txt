cmake_minimum_required(VERSION 3.27)
project(sepol)


# 对应: LOCAL_EXPORT_C_INCLUDES := $(LIBSEPOL)
# CMake 中通过 PUBLIC include 实现导出

# 对应: LOCAL_SRC_FILES := ...
set(LOCAL_SRC_FILES
        selinux/libsepol/src/assertion.c
        selinux/libsepol/src/avrule_block.c
        selinux/libsepol/src/avtab.c
        selinux/libsepol/src/boolean_record.c
        selinux/libsepol/src/booleans.c
        selinux/libsepol/src/conditional.c
        selinux/libsepol/src/constraint.c
        selinux/libsepol/src/context.c
        selinux/libsepol/src/context_record.c
        selinux/libsepol/src/debug.c
        selinux/libsepol/src/ebitmap.c
        selinux/libsepol/src/expand.c
        selinux/libsepol/src/handle.c
        selinux/libsepol/src/hashtab.c
        selinux/libsepol/src/hierarchy.c
        selinux/libsepol/src/ibendport_record.c
        selinux/libsepol/src/ibendports.c
        selinux/libsepol/src/ibpkey_record.c
        selinux/libsepol/src/ibpkeys.c
        selinux/libsepol/src/iface_record.c
        selinux/libsepol/src/interfaces.c
        selinux/libsepol/src/kernel_to_cil.c
        selinux/libsepol/src/kernel_to_common.c
        selinux/libsepol/src/kernel_to_conf.c
        selinux/libsepol/src/link.c
        selinux/libsepol/src/mls.c
        selinux/libsepol/src/module.c
        selinux/libsepol/src/module_to_cil.c
        selinux/libsepol/src/node_record.c
        selinux/libsepol/src/nodes.c
        selinux/libsepol/src/optimize.c
        selinux/libsepol/src/polcaps.c
        selinux/libsepol/src/policydb.c
        selinux/libsepol/src/policydb_convert.c
        selinux/libsepol/src/policydb_public.c
        selinux/libsepol/src/policydb_validate.c
        selinux/libsepol/src/port_record.c
        selinux/libsepol/src/ports.c
        selinux/libsepol/src/services.c
        selinux/libsepol/src/sidtab.c
        selinux/libsepol/src/symtab.c
        selinux/libsepol/src/user_record.c
        selinux/libsepol/src/users.c
        selinux/libsepol/src/util.c
        selinux/libsepol/src/write.c
        selinux/libsepol/cil/src/cil.c
        selinux/libsepol/cil/src/cil_binary.c
        selinux/libsepol/cil/src/cil_build_ast.c
        selinux/libsepol/cil/src/cil_copy_ast.c
        selinux/libsepol/cil/src/cil_deny.c
        selinux/libsepol/cil/src/cil_find.c
        selinux/libsepol/cil/src/cil_fqn.c
        selinux/libsepol/cil/src/cil_lexer.c
        selinux/libsepol/cil/src/cil_list.c
        selinux/libsepol/cil/src/cil_log.c
        selinux/libsepol/cil/src/cil_mem.c
        selinux/libsepol/cil/src/cil_parser.c
        selinux/libsepol/cil/src/cil_policy.c
        selinux/libsepol/cil/src/cil_post.c
        selinux/libsepol/cil/src/cil_reset_ast.c
        selinux/libsepol/cil/src/cil_resolve_ast.c
        selinux/libsepol/cil/src/cil_stack.c
        selinux/libsepol/cil/src/cil_strpool.c
        selinux/libsepol/cil/src/cil_symtab.c
        selinux/libsepol/cil/src/cil_tree.c
        selinux/libsepol/cil/src/cil_verify.c
        selinux/libsepol/cil/src/cil_write_ast.c
)

add_library(sepol STATIC ${LOCAL_SRC_FILES})


# 设置包含路径（对使用者也导出）
target_include_directories(sepol
        PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}/selinux/libsepol/src
        ${CMAKE_CURRENT_SOURCE_DIR}/selinux/libsepol/include
        ${CMAKE_CURRENT_SOURCE_DIR}/selinux/libsepol/cil/include
)
# 对应: LOCAL_CFLAGS := -Wno-unused-but-set-variable
# 以及架构条件
set(LOCAL_CFLAGS "-Wno-unused-but-set-variable")
if("${TARGET_ARCH}" STREQUAL "riscv64")
    list(APPEND LOCAL_CFLAGS "-DHAVE_REALLOCARRAY")
else ()
    message(STATUS "arch: ${TARGET_ARCH}")
endif()



if(CMAKE_SYSTEM_NAME STREQUAL "Android")
    # Android 编译
    message(STATUS "Building for Android")
    add_library(sepolicy SHARED export.cpp policydb.cpp sepolicy.cpp rules.cpp api.cpp statement.cpp logging.cpp util.cpp stream.cpp)
    target_include_directories(sepolicy
            PUBLIC
            ${CMAKE_CURRENT_SOURCE_DIR}/selinux/libsepol/src
            ${CMAKE_CURRENT_SOURCE_DIR}/selinux/libsepol/include
            ${CMAKE_CURRENT_SOURCE_DIR}/selinux/libsepol/cil/include
    )

    target_link_libraries(sepolicy sepol)

    add_library(preload SHARED preload.c)

elseif(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    # 普通 Linux 编译
    message(STATUS "Building for Linux arch ${TARGET_ARCH}")
    # 👇 关键：启用全静态链接


    set(CMAKE_EXE_LINKER_FLAGS "-static")

    add_executable(main main.cpp policydb.cpp sepolicy.cpp rules.cpp api.cpp statement.cpp logging.cpp util.cpp stream.cpp)

    target_compile_definitions(sepol PRIVATE HAVE_REALLOCARRAY)

    target_include_directories(main
            PUBLIC
            ${CMAKE_CURRENT_SOURCE_DIR}/selinux/libsepol/src
            ${CMAKE_CURRENT_SOURCE_DIR}/selinux/libsepol/include
            ${CMAKE_CURRENT_SOURCE_DIR}/selinux/libsepol/cil/include
    )
    target_link_libraries(main sepol)


else()
    message(STATUS "Building for other system: ${CMAKE_SYSTEM_NAME}, not support")
endif()

set(CMAKE_CXX_STANDARD 23)




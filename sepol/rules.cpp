
#include <vector>
#include "sepolicy.h"
#include "include/selinux.h"

using namespace std;

void sepolicy::magisk_rules() {
    // Temp suppress warnings
//    auto bak = log_cb.w;
//    log_cb.w = nop_log;



    // Prevent anything to change sepolicy except ourselves
    deny(ALL, "kernel", "security", "load_policy");
    type(SEPOL_PROC_DOMAIN, "domain");
    typeattribute(SEPOL_PROC_DOMAIN, "mlstrustedsubject");
    typeattribute(SEPOL_PROC_DOMAIN, "netdomain");
    typeattribute(SEPOL_PROC_DOMAIN, "bluetoothdomain");
//    type(SEPOL_FILE_TYPE, "file_type");
//    typeattribute(SEPOL_FILE_TYPE, "mlstrustedobject");
//    type(SEPOL_LOG_TYPE, "file_type");
//    typeattribute(SEPOL_LOG_TYPE, "mlstrustedobject");

    // Create unconstrained file type
//    allow(ALL, SEPOL_FILE_TYPE, "file", ALL);
//    allow(ALL, SEPOL_FILE_TYPE, "dir", ALL);
//    allow(ALL, SEPOL_FILE_TYPE, "fifo_file", ALL);
//    allow(ALL, SEPOL_FILE_TYPE, "chr_file", ALL);
//    allow(ALL, SEPOL_FILE_TYPE, "lnk_file", ALL);
//    allow(ALL, SEPOL_FILE_TYPE, "sock_file", ALL);


    // Only allow zygote to open log pipe
//    allow("zygote", SEPOL_LOG_TYPE, "fifo_file", "open");
//    allow("zygote", SEPOL_LOG_TYPE, "fifo_file", "read");
//     Allow all processes to output logs
//    allow("domain", SEPOL_LOG_TYPE, "fifo_file", "write");




    // Make our root domain unconstrained
    allow(SEPOL_PROC_DOMAIN, "fs_type", ALL, ALL);
    allow(SEPOL_PROC_DOMAIN, "dev_type", ALL, ALL);
    allow(SEPOL_PROC_DOMAIN, "file_type", ALL, ALL);
    allow(SEPOL_PROC_DOMAIN, "domain", ALL, ALL);
    allow(SEPOL_PROC_DOMAIN, "service_manager_type", ALL, ALL);
    allow(SEPOL_PROC_DOMAIN, "hwservice_manager_type", ALL, ALL);
    allow(SEPOL_PROC_DOMAIN, "vndservice_manager_type", ALL, ALL);
    allow(SEPOL_PROC_DOMAIN, "port_type", ALL, ALL);
    allow(SEPOL_PROC_DOMAIN, "node_type", ALL, ALL);
    allow(SEPOL_PROC_DOMAIN, "property_type", ALL, ALL);

    permissive(SEPOL_PROC_DOMAIN);  /* Just in case something is missing */

    // Allow us to do any ioctl
    if (impl->db->policyvers >= POLICYDB_VERSION_XPERMS_IOCTL) {
        allowxperm(SEPOL_PROC_DOMAIN, ALL, "blk_file", ALL);
        allowxperm(SEPOL_PROC_DOMAIN, ALL, "fifo_file", ALL);
        allowxperm(SEPOL_PROC_DOMAIN, ALL, "chr_file", ALL);
    }


    type(SEPOL_EXEC_TYPE, "file_type");
    typeattribute(SEPOL_EXEC_TYPE, "exec_type");

    //rzx+
    allow(SEPOL_EXEC_TYPE, "shell", ALL, ALL);
    allow("shell", SEPOL_EXEC_TYPE, ALL, ALL);
    allow("shell", SEPOL_PROC_DOMAIN, ALL, ALL);


    // Don't allow pesky processes to monitor audit deny logs when poking magisk daemon socket
//    dontaudit(ALL, SEPOL_PROC_DOMAIN, "unix_stream_socket", ALL);


    // Let everyone access tmpfs files (for SAR sbin overlay)
    allow(ALL, "tmpfs", "file", ALL);

    // Allow magiskinit daemon to handle mock selinuxfs
    allow("kernel", "tmpfs", "fifo_file", "write");

    // For relabelling files
    allow("rootfs", "labeledfs", "filesystem", "associate");
//    allow(SEPOL_FILE_TYPE, "pipefs", "filesystem", "associate");
//    allow(SEPOL_FILE_TYPE, "devpts", "filesystem", "associate");

    // Let init transit to SEPOL_PROC_DOMAIN
    allow("kernel", "kernel", "process", "setcurrent");
    allow("kernel", SEPOL_PROC_DOMAIN, "process", "dyntransition");

    // Let init run stuffs
    allow("kernel", SEPOL_PROC_DOMAIN, "fd", "use");
    allow("init", SEPOL_PROC_DOMAIN, "process", ALL);
    allow("init", "tmpfs", "file", "getattr");
    allow("init", "tmpfs", "file", "execute");

    // suRights
    allow(ALL, SEPOL_PROC_DOMAIN, "process", "sigchld");

    // allowLog
    allow("logd", SEPOL_PROC_DOMAIN, "dir", "search");
    allow("logd", SEPOL_PROC_DOMAIN, "file", "read");
    allow("logd", SEPOL_PROC_DOMAIN, "file", "open");
    allow("logd", SEPOL_PROC_DOMAIN, "file", "getattr");

    // dumpsys
    allow(ALL, SEPOL_PROC_DOMAIN, "fd", "use");
    allow(ALL, SEPOL_PROC_DOMAIN, "fifo_file", "write");
    allow(ALL, SEPOL_PROC_DOMAIN, "fifo_file", "read");
    allow(ALL, SEPOL_PROC_DOMAIN, "fifo_file", "open");
    allow(ALL, SEPOL_PROC_DOMAIN, "fifo_file", "getattr");

    // For mounting loop devices, mirrors, tmpfs
    allow("kernel", ALL, "file", "read");
    allow("kernel", ALL, "file", "write");

    // Allow all binder transactions
    allow(ALL, SEPOL_PROC_DOMAIN, "binder", ALL);

    // For changing file context
    allow("rootfs", "tmpfs", "filesystem", "associate");


    // Shut llkd up
    dontaudit("llkd", SEPOL_PROC_DOMAIN, "process", "ptrace");
//    dontaudit("llkd", SEPOL_CLIENT_DOMAIN, "process", "ptrace");

    // Allow update_engine/addon.d-v2 to run permissive on all ROMs
    permissive("update_engine");

#if 0
    // Remove all dontaudit in debug mode
    impl->strip_dontaudit();
#endif

//    log_cb.w = bak;
}

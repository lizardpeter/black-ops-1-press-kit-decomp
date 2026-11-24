typedef unsigned char   undefined;

typedef unsigned char    byte;
typedef unsigned int    dword;
typedef long long    longlong;
typedef unsigned char    uchar;
typedef unsigned int    uint;
typedef unsigned long    ulong;
typedef unsigned long long    ulonglong;
typedef unsigned char    undefined1;
typedef unsigned short    undefined2;
typedef unsigned int    undefined4;
typedef unsigned long long    undefined5;
typedef unsigned long long    undefined8;
typedef unsigned short    ushort;
typedef unsigned short    word;
typedef struct thread_state_hdr thread_state_hdr, *Pthread_state_hdr;

struct thread_state_hdr {
    dword flavor;
    dword count;
};

typedef struct cfstringStruct cfstringStruct, *PcfstringStruct;

struct cfstringStruct {
    dword field0_0x0;
    dword field1_0x4;
    pointer field2_0x8;
    int field3_0xc;
};

typedef unsigned short    wchar16;
typedef uint __uint32_t;

typedef __uint32_t __darwin_gid_t;

typedef longlong __int64_t;

typedef __int64_t __darwin_off_t;

typedef int __int32_t;

typedef __int32_t __darwin_suseconds_t;

typedef struct __darwin_pthread_handler_rec __darwin_pthread_handler_rec, *P__darwin_pthread_handler_rec;

struct __darwin_pthread_handler_rec {
    void (*__routine)(void *);
    void *__arg;
    struct __darwin_pthread_handler_rec *__next;
};

typedef struct _opaque_pthread_mutexattr_t _opaque_pthread_mutexattr_t, *P_opaque_pthread_mutexattr_t;

typedef struct _opaque_pthread_mutexattr_t __darwin_pthread_mutexattr_t;

struct _opaque_pthread_mutexattr_t {
    long __sig;
    char __opaque[8];
};

typedef __uint32_t __darwin_uid_t;

typedef struct _opaque_pthread_attr_t _opaque_pthread_attr_t, *P_opaque_pthread_attr_t;

struct _opaque_pthread_attr_t {
    long __sig;
    char __opaque[56];
};

typedef uint __darwin_natural_t;

typedef __darwin_natural_t __darwin_mach_port_name_t;

typedef __darwin_mach_port_name_t __darwin_mach_port_t;

typedef ushort __uint16_t;

typedef __uint16_t __darwin_mode_t;

typedef long __darwin_time_t;

typedef ulong __darwin_size_t;

typedef struct _opaque_pthread_condattr_t _opaque_pthread_condattr_t, *P_opaque_pthread_condattr_t;

typedef struct _opaque_pthread_condattr_t __darwin_pthread_condattr_t;

struct _opaque_pthread_condattr_t {
    long __sig;
    char __opaque[8];
};

typedef struct _opaque_pthread_cond_t _opaque_pthread_cond_t, *P_opaque_pthread_cond_t;

struct _opaque_pthread_cond_t {
    long __sig;
    char __opaque[40];
};

typedef struct _opaque_pthread_t _opaque_pthread_t, *P_opaque_pthread_t;

struct _opaque_pthread_t {
    long __sig;
    struct __darwin_pthread_handler_rec *__cleanup_stack;
    char __opaque[1168];
};

typedef struct _opaque_pthread_mutex_t _opaque_pthread_mutex_t, *P_opaque_pthread_mutex_t;

typedef struct _opaque_pthread_mutex_t __darwin_pthread_mutex_t;

struct _opaque_pthread_mutex_t {
    long __sig;
    char __opaque[56];
};

typedef struct _opaque_pthread_attr_t __darwin_pthread_attr_t;

typedef struct _opaque_pthread_t *__darwin_pthread_t;

typedef uchar __uint8_t;

typedef int __darwin_ct_rune_t;

typedef struct _opaque_pthread_cond_t __darwin_pthread_cond_t;

typedef long __darwin_ssize_t;

typedef __int32_t __darwin_pid_t;

typedef __uint32_t __darwin_socklen_t;

typedef ulong __darwin_pthread_key_t;

typedef __darwin_pthread_cond_t pthread_cond_t;

typedef __darwin_time_t time_t;

typedef struct timeval timeval, *Ptimeval;

struct timeval {
    __darwin_time_t tv_sec;
    __darwin_suseconds_t tv_usec;
};

typedef struct fd_set fd_set, *Pfd_set;

struct fd_set {
    __int32_t fds_bits[0];
};

typedef __darwin_mode_t mode_t;

typedef struct ipc_perm ipc_perm, *Pipc_perm;

typedef __darwin_uid_t uid_t;

typedef __darwin_gid_t gid_t;

typedef __int32_t key_t;

struct ipc_perm {
    uid_t uid;
    gid_t gid;
    uid_t cuid;
    gid_t cgid;
    mode_t mode;
    ushort _seq;
    key_t _key;
};

typedef int thread_state_flavor_t;

typedef __darwin_natural_t natural_t;

typedef natural_t *thread_state_t;

typedef __darwin_socklen_t socklen_t;

typedef natural_t mach_port_name_t;

typedef natural_t mach_port_right_t;

typedef struct mach_timebase_info mach_timebase_info, *Pmach_timebase_info;

typedef uint uint32_t;

struct mach_timebase_info {
    uint32_t numer;
    uint32_t denom;
};

typedef struct mach_timebase_info *mach_timebase_info_t;

typedef struct addrinfo addrinfo, *Paddrinfo;

typedef struct sockaddr sockaddr, *Psockaddr;

typedef __uint8_t sa_family_t;

struct addrinfo {
    int ai_flags;
    int ai_family;
    int ai_socktype;
    int ai_protocol;
    socklen_t ai_addrlen;
    char *ai_canonname;
    struct sockaddr *ai_addr;
    struct addrinfo *ai_next;
};

struct sockaddr {
    __uint8_t sa_len;
    sa_family_t sa_family;
    char sa_data[14];
};

typedef struct __sFILEX __sFILEX, *P__sFILEX;

struct __sFILEX {
};

typedef struct __sbuf __sbuf, *P__sbuf;

struct __sbuf {
    uchar *_base;
    int _size;
};

typedef struct __sFILE __sFILE, *P__sFILE;

typedef __darwin_off_t fpos_t;

typedef struct __sFILE FILE;

struct __sFILE {
    uchar *_p;
    int _r;
    int _w;
    short _flags;
    short _file;
    struct __sbuf _bf;
    int _lbfsize;
    void *_cookie;
    int (*_close)(void *);
    int (*_read)(void *, char *, int);
    fpos_t (*_seek)(void *, fpos_t, int);
    int (*_write)(void *, char *, int);
    struct __sbuf _ub;
    struct __sFILEX *_extra;
    int _ur;
    uchar _ubuf[3];
    uchar _nbuf[1];
    struct __sbuf _lb;
    int _blksize;
    fpos_t _offset;
};

typedef __darwin_pthread_mutexattr_t pthread_mutexattr_t;

typedef uint u_int;

typedef __darwin_pthread_condattr_t pthread_condattr_t;

typedef uint exception_mask_t;

typedef thread_state_flavor_t *exception_flavor_array_t;

typedef exception_mask_t *exception_mask_array_t;

typedef int integer_t;

typedef integer_t exception_data_type_t;

typedef int exception_behavior_t;

typedef exception_behavior_t *exception_behavior_array_t;

typedef exception_data_type_t *exception_data_t;

typedef int exception_type_t;

typedef __darwin_mach_port_t mach_port_t;

typedef mach_port_t exception_handler_t;

typedef exception_handler_t *exception_handler_array_t;

typedef mach_port_t thread_act_t;

typedef mach_port_t ipc_space_t;

typedef __darwin_ssize_t ssize_t;

typedef __darwin_pid_t pid_t;

typedef ulonglong uint64_t;

typedef struct timespec timespec, *Ptimespec;

struct timespec {
    __darwin_time_t tv_sec;
    long tv_nsec;
};

typedef ulong uintptr_t;

typedef __darwin_pthread_mutex_t pthread_mutex_t;

typedef __darwin_size_t size_t;

typedef natural_t mach_msg_type_number_t;

typedef uint mach_msg_bits_t;

typedef natural_t mach_msg_timeout_t;

typedef struct mach_msg_header_t mach_msg_header_t, *Pmach_msg_header_t;

typedef natural_t mach_msg_size_t;

typedef integer_t mach_msg_id_t;

struct mach_msg_header_t {
    mach_msg_bits_t msgh_bits;
    mach_msg_size_t msgh_size;
    mach_port_t msgh_remote_port;
    mach_port_t msgh_local_port;
    mach_msg_size_t msgh_reserved;
    mach_msg_id_t msgh_id;
};

typedef uint mach_msg_type_name_t;

typedef int kern_return_t;

typedef kern_return_t mach_msg_return_t;

typedef integer_t mach_msg_option_t;

typedef struct tm tm, *Ptm;

struct tm {
    int tm_sec;
    int tm_min;
    int tm_hour;
    int tm_mday;
    int tm_mon;
    int tm_year;
    int tm_wday;
    int tm_yday;
    int tm_isdst;
    long tm_gmtoff;
    char *tm_zone;
};

typedef __darwin_pthread_attr_t pthread_attr_t;

typedef __darwin_pthread_key_t pthread_key_t;

typedef __darwin_pthread_t pthread_t;

typedef struct __shmid_ds_new __shmid_ds_new, *P__shmid_ds_new;

typedef ushort shmatt_t;

struct __shmid_ds_new {
    struct ipc_perm shm_perm;
    size_t shm_segsz;
    pid_t shm_lpid;
    pid_t shm_cpid;
    shmatt_t shm_nattch;
    time_t shm_atime;
    time_t shm_dtime;
    time_t shm_ctime;
    void *shm_internal;
};

typedef struct _Unwind_Exception _Unwind_Exception, *P_Unwind_Exception;

typedef enum enum_4370 {
    _URC_NO_REASON=0,
    _URC_FOREIGN_EXCEPTION_CAUGHT=1,
    _URC_FATAL_PHASE2_ERROR=2,
    _URC_FATAL_PHASE1_ERROR=3,
    _URC_NORMAL_STOP=4,
    _URC_END_OF_STACK=5,
    _URC_HANDLER_FOUND=6,
    _URC_INSTALL_CONTEXT=7,
    _URC_CONTINUE_UNWIND=8
} enum_4370;

typedef enum enum_4370 _Unwind_Reason_Code;

struct _Unwind_Exception {
    uint64_t exception_class;
    void (*exception_cleanup)(_Unwind_Reason_Code, struct _Unwind_Exception *);
    uintptr_t private_1;
    uintptr_t private_2;
    uint32_t reserved[3];
};

typedef struct uuid_command uuid_command, *Puuid_command;

struct uuid_command {
    dword cmd;
    dword cmdsize;
    byte uuid[16];
};

typedef struct lc_str lc_str, *Plc_str;

struct lc_str {
    dword offset;
};

typedef struct thread_command thread_command, *Pthread_command;

typedef struct x86_THREAD_STATE32 x86_THREAD_STATE32, *Px86_THREAD_STATE32;

struct x86_THREAD_STATE32 {
    dword eax;
    dword ebx;
    dword ecx;
    dword edx;
    dword edi;
    dword esi;
    dword ebp;
    dword esp;
    dword ss;
    dword eflags;
    dword eip;
    dword cs;
    dword ds;
    dword es;
    dword fs;
    dword gs;
};

struct thread_command {
    dword cmd;
    dword cmdsize;
    struct thread_state_hdr threadStateHeader;
    struct x86_THREAD_STATE32 threadState;
};

typedef struct dylib dylib, *Pdylib;

struct dylib {
    struct lc_str name;
    dword timestamp;
    dword current_version;
    dword compatibility_version;
};

typedef struct section section, *Psection;

struct section {
    char sectname[16];
    char segname[16];
    dword addr;
    dword size;
    dword offset;
    dword align;
    dword reloff;
    dword nrelocs;
    dword flags;
    dword reserved1;
    dword reserved2;
};

typedef struct nlist nlist, *Pnlist;

struct nlist {
    dword n_strx;
    byte n_type;
    byte n_sect;
    word n_desc;
    dword n_value;
};

typedef struct twolevel_hints_command twolevel_hints_command, *Ptwolevel_hints_command;

struct twolevel_hints_command {
    dword cmd;
    dword cmdsize;
    dword offset;
    dword nhints;
};

typedef struct dysymtab_command dysymtab_command, *Pdysymtab_command;

struct dysymtab_command {
    dword cmd;
    dword cmdsize;
    dword ilocalsym;
    dword nlocalsym;
    dword iextdefsym;
    dword nextdefsym;
    dword iundefsym;
    dword nundefsym;
    dword tocoff;
    dword ntoc;
    dword modtaboff;
    dword nmodtab;
    dword extrefsymoff;
    dword nextrefsyms;
    dword indirectsymoff;
    dword nindirectsyms;
    dword extreloff;
    dword nextrel;
    dword locreloff;
    dword nlocrel;
};

typedef struct segment_command segment_command, *Psegment_command;

struct segment_command {
    dword cmd;
    dword cmdsize;
    char segname[16];
    dword vmaddr;
    dword vmsize;
    dword fileoff;
    dword filesize;
    dword maxprot;
    dword initprot;
    dword nsects;
    dword flags;
};

typedef struct dylib_command dylib_command, *Pdylib_command;

struct dylib_command {
    dword cmd;
    dword cmdsize;
    struct dylib dylib;
};

typedef struct symtab_command symtab_command, *Psymtab_command;

struct symtab_command {
    dword cmd;
    dword cmdsize;
    dword symoff;
    dword nsyms;
    dword stroff;
    dword strsize;
};

typedef struct mach_header mach_header, *Pmach_header;

struct mach_header {
    dword magic;
    dword cputype;
    dword cpusubtype;
    dword filetype;
    dword ncmds;
    dword sizeofcmds;
    dword flags;
};

typedef struct dylinker_command dylinker_command, *Pdylinker_command;

struct dylinker_command {
    dword cmd;
    dword cmdsize;
    struct lc_str name;
};


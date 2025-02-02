//! Abstraction of POSIX constructs with a safe API

use access_control_list::AccessControlListError;
use barrier::BarrierCreationError;
use clock::ClockError;
use directory::DirectoryError;
use elkodon_bb_elementary::enum_gen;
use file::FileError;
use file_lock::FileLockError;
use group::GroupError;
use memory_lock::MemoryLockError;
use mutex::MutexError;
use process::ProcessError;
use read_write_mutex::ReadWriteMutexError;
use semaphore::SemaphoreError;
use shared_memory::SharedMemoryCreationError;
use signal::SignalError;
use thread::ThreadError;
use unix_datagram_socket::UnixDatagramError;
use user::UserError;

pub mod access_control_list;
pub mod access_mode;
pub mod adaptive_wait;
pub mod barrier;
pub mod clock;
pub mod condition_variable;
pub mod config;
pub mod creation_mode;
pub mod udp_socket;
#[macro_use]
pub mod handle_errno;
pub mod directory;
pub mod file;
pub mod file_descriptor;
pub mod file_descriptor_set;
pub mod file_lock;
pub mod file_type;
pub mod group;
pub mod memory;
pub mod memory_lock;
pub mod message_queue;
pub mod metadata;
pub mod mutex;
pub mod ownership;
pub mod permission;
pub mod process;
pub mod read_write_mutex;
pub mod scheduler;
pub mod semaphore;
pub mod shared_memory;
pub mod signal;
pub mod socket_ancillary;
pub mod system_configuration;
pub mod thread;
pub mod unique_system_id;
pub mod unix_datagram_socket;
pub mod unmovable_ipc_handle;
pub mod user;

enum_gen! {Error
  generalization:
    AccessControlList <= AccessControlListError,
    Barrier <= BarrierCreationError,
    Clock <= ClockError,
    Directory <= DirectoryError,
    File <= FileError,
    FileLock <= FileLockError,
    Group <= GroupError,
    MemoryLock <= MemoryLockError,
    Mutex <= MutexError,
    Process <= ProcessError,
    ReadWriteMutex <= ReadWriteMutexError,
    Semaphore <= SemaphoreError,
    SharedMemory <= SharedMemoryCreationError,
    Signal <= SignalError,
    Thread <= ThreadError,
    User <= UserError,
    UnixDatagramSocket <= UnixDatagramError
}

//! Provides a [`FileDescriptor`] abstraction which takes the ownership of low-level POSIX
//! file descriptors and the [`FileDescriptorBased`] & [`FileDescriptorManagement`] traits
//! which provide advanced functionalities to all [`FileDescriptorBased`] constructs.
//!
//! # Examples
//! ## Use [`FileDescriptorManagement`] to extend a type
//!
//! ```
//! use elkodon_bb_posix::file_descriptor::*;
//!
//! // required for FileDescriptorManagement
//! #[derive(Debug)]
//! pub struct SomeConstructBasedOnFileDescriptor {
//!   fd: FileDescriptor
//! }
//!
//! // implement FileDescriptorBased trait
//! impl FileDescriptorBased for SomeConstructBasedOnFileDescriptor {
//!     fn file_descriptor(&self) -> &FileDescriptor {
//!         &self.fd
//!     }
//! }
//!
//!
//! // auto implement the FileDescriptorManagement trait to gain more file descriptor management
//! // features
//! impl FileDescriptorManagement for SomeConstructBasedOnFileDescriptor {}
//! ```
//!
//! ## Work with [`FileDescriptorManagement`]
//!
//! ```no_run
//! use elkodon_bb_system_types::file_path::FilePath;
//! use elkodon_bb_container::semantic_string::SemanticString;
//! use elkodon_bb_posix::file_descriptor::*;
//! use elkodon_bb_posix::file::*;
//! use elkodon_bb_posix::access_control_list::*;
//! use elkodon_bb_posix::ownership::*;
//! use elkodon_bb_posix::user::UserExt;
//! use elkodon_bb_posix::group::GroupExt;
//!
//! let file_name = FilePath::new(b"/tmp/someFile").unwrap();
//! let mut file = FileBuilder::new(&file_name).creation_mode(CreationMode::PurgeAndCreate)
//!                              .create().expect("failed to create file");
//!
//! println!("owner: {:?}", file.ownership().unwrap());
//! println!("permission: {}", file.permission().unwrap());
//! println!("metadata: {:?}", file.metadata().unwrap());
//!
//! // set new owner
//! file.set_ownership(OwnershipBuilder::new()
//!         .uid("testuser1".as_user().unwrap().uid())
//!         .gid("testgroup1".as_group().unwrap().gid()).create());
//!
//! // set new permissions
//! file.set_permission(Permission::ALL);
//!
//! // set some new ACLs
//! let mut acl = file.access_control_list().expect("failed to get acl");
//! acl.add_user("testUser2".as_user().unwrap().uid(), AclPermission::Read)
//!     .expect("failed to add user");
//! file.set_access_control_list(&acl);
//! ```

use std::fmt::Debug;

use crate::access_control_list::*;
use crate::config::EINTR_REPETITIONS;
use crate::file::*;
use crate::metadata::Metadata;
use crate::ownership::*;
use crate::permission::{Permission, PermissionExt};
use elkodon_bb_log::{error, fail, fatal_panic};
use elkodon_pal_posix::posix::errno::Errno;
use elkodon_pal_posix::*;

/// Represents a FileDescriptor in a POSIX system. Contains always a value greater or equal zero,
/// a valid file descriptor. It takes the ownership of the provided file descriptor and calls
/// [`posix::close`] on destruction.
///
/// # Example
///
/// ```ignore
/// use elkodon_bb_posix::file_descriptor::*;
///
/// let valid_fd = FileDescriptor::new(2);
/// let invalid_fd = FileDescriptor::new(-4);
///
/// println!("Created FD: {:?}", valid_fd.unwrap());
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct FileDescriptor {
    value: i32,
    is_owned: bool,
}

impl Clone for FileDescriptor {
    fn clone_from(&mut self, source: &Self) {
        self.close();
        *self = source.clone();
    }

    fn clone(&self) -> Self {
        let fd_clone = unsafe { posix::dup(self.value) };
        if fd_clone < 0 {
            let msg = "Unable to clone file descriptor";
            match Errno::get() {
                Errno::EMFILE => {
                    fatal_panic!(from self, "{} since the maximum amount of open file descriptors for the process is reached.", msg)
                }
                v => fatal_panic!(from self, "{} since an unknown error occurred ({}).", msg, v),
            }
        }

        Self {
            value: fd_clone,
            is_owned: true,
        }
    }
}

impl FileDescriptor {
    /// Creates a FileDescriptor which does not hold the ownership of the file descriptor and will
    /// not call [`posix::close`] on destruction.
    pub fn non_owning_new(value: i32) -> Option<FileDescriptor> {
        if value < 0 {
            return None;
        }

        Some(FileDescriptor {
            value,
            is_owned: false,
        })
    }

    /// Creates a new FileDescriptor. If the value is smaller than zero it returns [`None`].
    pub fn new(value: i32) -> Option<FileDescriptor> {
        if value < 0 {
            return None;
        }

        if unsafe { posix::fcntl2(value, posix::F_GETFD) } < 0 {
            return None;
        }

        Some(FileDescriptor {
            value,
            is_owned: true,
        })
    }

    /// Creates a new FileDescriptor.
    ///
    /// # Safety
    ///
    ///  * it must be a valid file descriptor
    ///
    pub unsafe fn new_unchecked(value: i32) -> FileDescriptor {
        FileDescriptor {
            value,
            is_owned: true,
        }
    }

    /// Returns the underlying value of the FileDescriptor
    ///
    /// # Safety
    ///
    ///  * the user shall not store the value in a variable otherwise lifetime issues may be
    ///    encountered
    ///  * do not manually close the file descriptor with a sys call
    ///
    pub unsafe fn native_handle(&self) -> i32 {
        self.value
    }

    fn close(&mut self) {
        let mut counter = 0;
        loop {
            if unsafe { posix::close(self.value) } == 0 {
                break;
            }

            match Errno::get() {
                Errno::EBADF => {
                    fatal_panic!(from self, "This should never happen! Unable to close file due to an invalid file-descriptor.");
                }
                Errno::EINTR => {
                    counter += 1;
                    if counter > EINTR_REPETITIONS {
                        error!(from self, "Unable to close file since too many interrupt signals were received.");
                    }
                }
                Errno::EIO => {
                    error!(from self, "Unable to close file due to an I/O error.");
                    counter += 1;
                }
                v => {
                    fatal_panic!(from self, "This should never happen! Unable to close file since an unknown error occurred ({}).", v);
                }
            }

            if counter > EINTR_REPETITIONS {
                error!(from self, "Tried {} times to close the file but failed.", counter);
            }
        }
    }
}

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        if self.is_owned {
            self.close()
        }
    }
}

/// Every construct which is based on some [`FileDescriptor`] can implement this trait to gain
/// extended [`FileDescriptorManagement`] features.
pub trait FileDescriptorBased {
    /// Returns the file descriptor of the underlying construct
    fn file_descriptor(&self) -> &FileDescriptor;
}

impl FileDescriptorBased for FileDescriptor {
    fn file_descriptor(&self) -> &FileDescriptor {
        self
    }
}

impl FileDescriptorManagement for FileDescriptor {}

/// Provides additional feature for every file descriptor based construct like
///  * ownership handling, [`ownership`](FileDescriptorManagement::ownership()),
///                        [`set_ownership`](FileDescriptorManagement::set_ownership())
///  * permission handling, [`permission`](FileDescriptorManagement::permission()),
///                         [`set_permission`](FileDescriptorManagement::set_permission())
///  * truncate size, [`truncate`](FileDescriptorManagement::truncate())
///  * accessing extended stats via [`Metadata`], [`metadata`](FileDescriptorManagement::metadata())
///  * access control list handling,
///         [`access_control_list`](FileDescriptorManagement::access_control_list())
///         [`set_access_control_list`](FileDescriptorManagement::set_access_control_list())
///
pub trait FileDescriptorManagement: FileDescriptorBased + Debug + Sized {
    /// Returns the current user and group owner of the file descriptor
    fn ownership(&self) -> Result<Ownership, FileStatError> {
        let attr =
            fail!(from self, when File::acquire_attributes(self), "Unable to read file owner.");
        Ok(OwnershipBuilder::new()
            .uid(attr.st_uid)
            .gid(attr.st_gid)
            .create())
    }

    /// Sets a new user and group owner
    fn set_ownership(&mut self, ownership: Ownership) -> Result<(), FileSetOwnerError> {
        Ok(
            fail!(from self, when File::set_ownership(self, ownership.uid(), ownership.gid()),  "Unable to set owner of the file."),
        )
    }

    /// Returns the current permission of the file descriptor
    fn permission(&self) -> Result<Permission, FileStatError> {
        Ok(
            fail!(from self, when File::acquire_attributes(self), "Unable to read permissions.")
                .st_mode
                .as_permission(),
        )
    }

    /// Sets new permissions
    fn set_permission(&mut self, permission: Permission) -> Result<(), FileSetPermissionError> {
        fail!(from self, when File::set_permission(self, permission),
                    "Unable to update permission.");
        Ok(())
    }

    /// Truncates to the file descriptor corresponding construct
    fn truncate(&mut self, size: usize) -> Result<(), FileTruncateError> {
        fail!(from self, when File::truncate(self, size),
                    "Unable to truncate to {}.", size);
        Ok(())
    }

    /// Requires all available [`Metadata`] for the file descriptor
    fn metadata(&self) -> Result<Metadata, FileStatError> {
        Ok(Metadata::create(
            &fail!(from self, when File::acquire_attributes(self),
                    "Unable to acquire attributes to create Metadata."),
        ))
    }

    /// Returns the current access control list
    fn access_control_list(
        &self,
    ) -> Result<AccessControlList, AccessControlListCreationFromFdError> {
        AccessControlList::from_file_descriptor(unsafe { self.file_descriptor().native_handle() })
    }

    /// Sets a new access control list
    fn set_access_control_list(
        &self,
        acl: &AccessControlList,
    ) -> Result<(), AccessControlListApplyError> {
        acl.apply_to_file_descriptor(unsafe { self.file_descriptor().native_handle() })
    }
}

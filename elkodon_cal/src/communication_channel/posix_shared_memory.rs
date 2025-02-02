//! [`CommunicationChannel`] which is able to send and receive only [`usize`] values
//! (**except** [`usize::MAX`]).
//!
//! It uses internally a [`DynamicStorage`] and [`SafelyOverflowingIndexQueue`].
pub use crate::communication_channel::*;

use crate::dynamic_storage::{
    self, DynamicStorage, DynamicStorageBuilder, DynamicStorageCreateError, DynamicStorageOpenError,
};
use crate::named_concept::*;
use elkodon_bb_elementary::relocatable_container::*;
use elkodon_bb_lock_free::spsc::safely_overflowing_index_queue::*;
use elkodon_bb_log::fail;

type SharedMemory = dynamic_storage::posix_shared_memory::Storage<Management>;
type SharedMemoryBuilder = <SharedMemory as DynamicStorage<Management>>::Builder;

#[derive(Debug)]
pub struct Channel {}

impl NamedConceptMgmt for Channel {
    type Configuration = Configuration;

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
        SharedMemory::does_exist_cfg(name, &(*cfg).into())
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
        SharedMemory::list_cfg(&(*cfg).into())
    }

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
        SharedMemory::remove_cfg(name, &(*cfg).into())
    }
}

impl CommunicationChannel<usize> for Channel {
    type Sender = Sender;
    type Receiver = Receiver;
    type Creator = Creator;
    type Connector = Connector;

    fn does_support_safe_overflow() -> bool {
        true
    }

    fn has_configurable_buffer_size() -> bool {
        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Configuration {
    suffix: FileName,
    path_hint: Path,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            suffix: DEFAULT_SUFFIX,
            path_hint: DEFAULT_PATH_HINT,
        }
    }
}

impl From<Configuration> for dynamic_storage::posix_shared_memory::Configuration {
    fn from(value: Configuration) -> Self {
        Self::default()
            .suffix(value.suffix)
            .path_hint(value.path_hint)
    }
}

impl NamedConceptConfiguration for Configuration {
    fn suffix(mut self, value: FileName) -> Self {
        self.suffix = value;
        self
    }

    fn path_hint(mut self, value: Path) -> Self {
        self.path_hint = value;
        self
    }

    fn get_suffix(&self) -> &FileName {
        &self.suffix
    }

    fn get_path_hint(&self) -> &Path {
        &self.path_hint
    }
}

#[derive(Debug)]
pub struct Creator {
    pub(crate) channel_name: FileName,
    enable_safe_overflow: bool,
    buffer_size: usize,
    config: Configuration,
}

impl NamedConceptBuilder<Channel> for Creator {
    fn new(channel_name: &FileName) -> Self {
        Self {
            channel_name: *channel_name,
            enable_safe_overflow: false,
            buffer_size: DEFAULT_RECEIVER_BUFFER_SIZE,
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = *config;
        self
    }
}

impl CommunicationChannelCreator<usize, Channel> for Creator {
    fn enable_safe_overflow(mut self) -> Self {
        self.enable_safe_overflow = true;
        self
    }

    fn buffer_size(mut self, value: usize) -> Self {
        self.buffer_size = value;
        self
    }

    fn create_receiver(self) -> Result<Receiver, CommunicationChannelCreateError> {
        let msg = "Unable to create communication channel";
        let shared_memory = match SharedMemoryBuilder::new(&self.channel_name)
            .config(&self.config.into())
            .supplementary_size(SafelyOverflowingIndexQueue::const_memory_size(
                self.buffer_size,
            ))
            .create_and_initialize(
                Management {
                    enable_safe_overflow: self.enable_safe_overflow,
                    index_queue: unsafe {
                        RelocatableSafelyOverflowingIndexQueue::new_uninit(self.buffer_size)
                    },
                },
                |mgmt, allocator| unsafe { mgmt.index_queue.init(allocator).is_ok() },
            ) {
            Ok(s) => s,
            Err(DynamicStorageCreateError::AlreadyExists) => {
                fail!(from self, with CommunicationChannelCreateError::AlreadyExists,
                    "{} since a channel with that name already exists.", msg);
            }
            Err(v) => {
                fail!(from self, with CommunicationChannelCreateError::InternalFailure,
                    "{} due to an internal failure ({:?})", msg, v);
            }
        };

        Ok(Receiver { shared_memory })
    }
}

#[derive(Debug)]
pub struct Connector {
    pub(crate) channel_name: FileName,
    config: Configuration,
}

impl NamedConceptBuilder<Channel> for Connector {
    fn new(channel_name: &FileName) -> Self {
        Self {
            channel_name: *channel_name,
            config: Configuration::default(),
        }
    }

    fn config(mut self, config: &Configuration) -> Self {
        self.config = *config;
        self
    }
}

impl CommunicationChannelConnector<usize, Channel> for Connector {
    fn try_open_sender(self) -> Result<Sender, CommunicationChannelOpenError> {
        let msg = "Unable to try open communication channel";

        match SharedMemoryBuilder::new(&self.channel_name)
            .config(&self.config.into())
            .try_open()
        {
            Ok(shared_memory) => Ok(Sender { shared_memory }),
            Err(DynamicStorageOpenError::DoesNotExist)
            | Err(DynamicStorageOpenError::InitializationNotYetFinalized) => {
                Err(CommunicationChannelOpenError::DoesNotExist)
            }
            Err(v) => {
                fail!(from self, with CommunicationChannelOpenError::InternalFailure,
                    "{} since an internal failure occurred ({:?}).", msg, v);
            }
        }
    }

    fn open_sender(self) -> Result<Sender, CommunicationChannelOpenError> {
        let msg = "Unable to open communication channel";
        let origin = format!("{:?}", self);
        match self.try_open_sender() {
            Ok(s) => Ok(s),
            Err(CommunicationChannelOpenError::DoesNotExist) => {
                fail!(from origin, with CommunicationChannelOpenError::DoesNotExist,
                    "{} since the channel does not exist.", msg);
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Management {
    index_queue: RelocatableSafelyOverflowingIndexQueue,
    enable_safe_overflow: bool,
}

#[derive(Debug)]
pub struct Receiver {
    shared_memory: SharedMemory,
}

impl NamedConcept for Receiver {
    fn name(&self) -> &FileName {
        self.shared_memory.name()
    }
}

impl Receiver {
    fn management(&self) -> &Management {
        self.shared_memory.get()
    }
}

impl CommunicationChannelParticipant for Receiver {
    fn does_enable_safe_overflow(&self) -> bool {
        self.management().enable_safe_overflow
    }
}

impl CommunicationChannelReceiver<usize> for Receiver {
    fn buffer_size(&self) -> usize {
        self.management().index_queue.capacity()
    }

    fn receive(&self) -> Result<Option<usize>, CommunicationChannelReceiveError> {
        Ok(unsafe { self.management().index_queue.pop() })
    }
}

#[derive(Debug)]
pub struct Sender {
    shared_memory: SharedMemory,
}

impl Sender {
    fn management(&self) -> &Management {
        self.shared_memory.get()
    }
}

impl CommunicationChannelParticipant for Sender {
    fn does_enable_safe_overflow(&self) -> bool {
        self.management().enable_safe_overflow
    }
}

impl NamedConcept for Sender {
    fn name(&self) -> &FileName {
        self.shared_memory.name()
    }
}

impl CommunicationChannelSender<usize> for Sender {
    fn send(&self, value: &usize) -> Result<Option<usize>, CommunicationChannelSendError> {
        match self.try_send(value) {
            Err(CommunicationChannelSendError::ReceiverCacheIsFull) => {
                fail!(from self, with CommunicationChannelSendError::ReceiverCacheIsFull,
                    "Unable to send data since the corresponding receiver cache is full.");
            }
            Err(e) => Err(e),
            Ok(s) => Ok(s),
        }
    }

    fn try_send(&self, value: &usize) -> Result<Option<usize>, CommunicationChannelSendError> {
        if !self.management().enable_safe_overflow && self.management().index_queue.is_full() {
            return Err(CommunicationChannelSendError::ReceiverCacheIsFull);
        }

        Ok(unsafe { self.management().index_queue.push(*value) })
    }
}

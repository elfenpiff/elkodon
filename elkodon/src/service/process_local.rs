//! # Example
//!
//! ```
//! use elkodon::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service_name = ServiceName::new(b"My/Funk/ServiceName")?;
//!
//! // use `process_local` as communication variant
//! let service = process_local::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let publisher = service.publisher().create()?;
//! let subscriber = service.subscriber().create()?;
//!
//! # Ok(())
//! # }
//! ```

use crate::port::event_id::EventId;
use crate::service::dynamic_config::DynamicConfig;
use elkodon_cal::shm_allocator::pool_allocator::PoolAllocator;
use elkodon_cal::*;

use super::ServiceState;

/// Defines a process local or single address space communication setup.
#[derive(Debug)]
pub struct Service<'config> {
    state: ServiceState<
        'config,
        static_storage::process_local::Storage,
        dynamic_storage::process_local::Storage<DynamicConfig>,
    >,
}

impl<'config> crate::service::Service for Service<'config> {
    type Type<'b> = Service<'b>;
}

impl<'config> crate::service::Details<'config> for Service<'config> {
    type StaticStorage = static_storage::process_local::Storage;
    type ConfigSerializer = serialize::toml::Toml;
    type DynamicStorage = dynamic_storage::process_local::Storage<DynamicConfig>;
    type ServiceNameHasher = hash::sha1::Sha1;
    type SharedMemory = shared_memory::process_local::Memory<PoolAllocator>;
    type Connection = zero_copy_connection::process_local::Connection;
    type Event = event::process_local::Event<EventId>;

    fn from_state(state: ServiceState<'config, Self::StaticStorage, Self::DynamicStorage>) -> Self {
        Self { state }
    }

    fn state(&self) -> &ServiceState<'config, Self::StaticStorage, Self::DynamicStorage> {
        &self.state
    }

    fn state_mut(
        &mut self,
    ) -> &mut ServiceState<'config, Self::StaticStorage, Self::DynamicStorage> {
        &mut self.state
    }
}

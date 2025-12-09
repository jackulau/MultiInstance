//! Core module - Application state, instance management, and resource monitoring

mod app_state;
mod instance;
mod monitor;
mod process;
pub mod profile;
pub mod resource;
pub mod settings;

pub use app_state::AppState;
pub use instance::{Instance, InstanceConfig, InstanceId, InstanceStatus};
pub use profile::{Profile, ProfileId};
pub use resource::ResourceLimits;
pub use settings::Settings;

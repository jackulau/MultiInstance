//! Dialog windows

pub mod confirm;
pub mod edit_instance;
pub mod edit_profile;
pub mod instance_details;
pub mod new_instance;
pub mod new_profile;

use crate::core::{InstanceId, ProfileId};
use std::sync::Arc;

/// State for dialog windows
#[derive(Default)]
pub enum DialogState {
    #[default]
    None,
    NewInstance,
    EditInstance(InstanceId),
    NewProfile,
    EditProfile(ProfileId),
    InstanceDetails(InstanceId),
    Confirm {
        title: String,
        message: String,
        on_confirm: Arc<dyn Fn() + Send + Sync>,
    },
}

impl Clone for DialogState {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::NewInstance => Self::NewInstance,
            Self::EditInstance(id) => Self::EditInstance(*id),
            Self::NewProfile => Self::NewProfile,
            Self::EditProfile(id) => Self::EditProfile(*id),
            Self::InstanceDetails(id) => Self::InstanceDetails(*id),
            Self::Confirm {
                title,
                message,
                on_confirm,
            } => Self::Confirm {
                title: title.clone(),
                message: message.clone(),
                on_confirm: Arc::clone(on_confirm),
            },
        }
    }
}

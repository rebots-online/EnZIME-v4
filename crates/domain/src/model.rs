use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelLifecycleState {
    Available,
    Downloading,
    Verifying,
    Installed,
    Active,
    Quarantined,
    Removing,
}

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Representation of an activity that a [`User`] is performing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Activity {
    pub application_id: Option<ApplicationId>,
    pub assets: Option<ActivityAssets>,
    pub details: Option<String>,
    pub flags: Option<ActivityFlags>,
    pub instance: Option<bool>,
    #[serde(default = "ActivityType::default", rename = "type")]
    pub kind: ActivityType,
    pub name: String,
    pub party: Option<ActivityParty>,
    pub secrets: Option<ActivitySecrets>,
    pub state: Option<String>,
    pub timestamps: Option<ActivityTimestamps>,
    pub url: Option<String>,
}

#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct ApplicationId(pub u64);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActivityAssets {
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
}

bitflags! {
    #[derive(Serialize)]
    pub struct ActivityFlags: u64 {
        const INSTANCE = 0b001;
        const JOIN = 0b010;
        const SPECTATE = 0b011;
        const JOIN_REQUEST = 0b100;
        const SYNC = 0b101;
        const PLAY = 0b110;
    }
}

impl<'de> Deserialize<'de> for ActivityFlags {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = u64::deserialize(deserializer)?;
        Ok(ActivityFlags::from_bits_truncate(raw))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActivityParty {
    pub id: Option<String>,
    pub size: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActivitySecrets {
    pub join: Option<String>,
    #[serde(rename = "match")]
    pub match_: Option<String>,
    pub spectate: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum ActivityType {
    Playing = 0,
    Streaming = 1,
    Listening = 2,
    Unknown = 3,
    Custom = 4,
}

impl Default for ActivityType {
    fn default() -> Self {
        ActivityType::Playing
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Presence {
    #[serde(rename = "game")]
    pub activity: Option<Activity>,
    pub last_modified: Option<u64>,
    pub nick: Option<String>,
    pub status: OnlineStatus,
    pub user: User,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub enum OnlineStatus {
    #[serde(rename = "Dnd")]
    DoNotDisturb,
    #[serde(rename = "Idle")]
    Idle,
    #[serde(rename = "Invisible")]
    Invisible,
    #[serde(rename = "Offline")]
    Offline,
    #[serde(rename = "Online")]
    Online,
}

impl Default for OnlineStatus {
    fn default() -> OnlineStatus {
        OnlineStatus::Online
    }
}
#[derive(
    Copy, Clone, Default, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct UserId(pub u64);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: UserId,
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: Option<bool>,
    pub discriminator: Option<String>,
    #[serde(rename = "username")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActivityTimestamps {
    pub end: Option<u64>,
    pub start: Option<u64>,
}

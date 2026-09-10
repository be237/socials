use std::path::PathBuf;
use std::sync::Mutex;

use std::collections::HashMap;

use grammers_session::types::{DcOption, PeerId, PeerInfo, UpdateState, UpdatesState};
use grammers_session::{Session, SessionData};

#[derive(serde::Serialize, serde::Deserialize)]
struct SerializableSessionData {
    home_dc: i32,
    dc_options: HashMap<i32, DcOption>,
    peer_infos: HashMap<PeerId, PeerInfo>,
    updates_state: UpdatesState,
}

/// A session that persists `SessionData` to a JSON file.
pub struct FileSession {
    data: Mutex<SessionData>,
    path: PathBuf,
}

#[derive(Debug)]
pub enum FileSessionError {
    Io(std::io::Error),
    Serde(serde_json::Error),
}

impl std::error::Error for FileSessionError {}

impl std::fmt::Display for FileSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Serde(e) => write!(f, "serde error: {}", e),
        }
    }
}

impl From<std::io::Error> for FileSessionError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for FileSessionError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

impl FileSession {
    pub fn new(path: PathBuf) -> Result<Self, FileSessionError> {
        let data = if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let serializable: SerializableSessionData = serde_json::from_str(&content)?;
            SessionData {
                home_dc: serializable.home_dc,
                dc_options: serializable.dc_options,
                peer_infos: serializable.peer_infos,
                updates_state: serializable.updates_state,
            }
        } else {
            SessionData::default()
        };

        Ok(Self {
            data: Mutex::new(data),
            path,
        })
    }

    fn save(&self) -> Result<(), FileSessionError> {
        let data = self.data.lock().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "session lock poisoned")
        })?;
        let serializable = SerializableSessionData {
            home_dc: data.home_dc,
            dc_options: data.dc_options.clone(),
            peer_infos: data.peer_infos.clone(),
            updates_state: data.updates_state.clone(),
        };
        let json = serde_json::to_string_pretty(&serializable)?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, json)?;
        Ok(())
    }
}

impl Session for FileSession {
    type Error = FileSessionError;

    fn home_dc_id(&self) -> Result<i32, FileSessionError> {
        Ok(self
            .data
            .lock()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "lock"))?
            .home_dc)
    }

    fn set_home_dc_id(&self, dc_id: i32) -> grammers_session::BoxFuture<'_, Result<(), FileSessionError>> {
        Box::pin(async move {
            self.data
                .lock()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "lock"))?
                .home_dc = dc_id;
            self.save()
        })
    }

    fn dc_option(&self, dc_id: i32) -> Result<Option<DcOption>, FileSessionError> {
        Ok(self
            .data
            .lock()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "lock"))?
            .dc_options
            .get(&dc_id)
            .cloned())
    }

    fn set_dc_option(&self, dc_option: &DcOption) -> grammers_session::BoxFuture<'_, Result<(), FileSessionError>> {
        let dc_option = dc_option.clone();
        Box::pin(async move {
            self.data
                .lock()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "lock"))?
                .dc_options
                .insert(dc_option.id, dc_option);
            self.save()
        })
    }

    fn peer(&self, peer: PeerId) -> grammers_session::BoxFuture<'_, Result<Option<PeerInfo>, FileSessionError>> {
        Box::pin(async move {
            Ok(self
                .data
                .lock()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "lock"))?
                .peer_infos
                .get(&peer)
                .cloned())
        })
    }

    fn cache_peer(&self, peer: PeerInfo) -> grammers_session::BoxFuture<'_, Result<(), FileSessionError>> {
        Box::pin(async move {
            self.data
                .lock()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "lock"))?
                .peer_infos
                .entry(peer.id())
                .or_insert_with(|| peer.clone())
                .extend_info(&peer);
            self.save()
        })
    }

    fn updates_state(&self) -> grammers_session::BoxFuture<'_, Result<UpdatesState, FileSessionError>> {
        Box::pin(async move {
            Ok(self
                .data
                .lock()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "lock"))?
                .updates_state
                .clone())
        })
    }

    fn set_update_state(
        &self,
        update: UpdateState,
    ) -> grammers_session::BoxFuture<'_, Result<(), FileSessionError>> {
        Box::pin(async move {
            let mut data = self
                .data
                .lock()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "lock"))?;

            match update {
                UpdateState::All(updates_state) => {
                    data.updates_state = updates_state;
                }
                UpdateState::Primary { pts, date, seq } => {
                    data.updates_state.pts = pts;
                    data.updates_state.date = date;
                    data.updates_state.seq = seq;
                }
                UpdateState::Secondary { qts } => {
                    data.updates_state.qts = qts;
                }
                UpdateState::Channel { id, pts } => {
                    data.updates_state.channels.retain(|c| c.id != id);
                    data.updates_state
                        .channels
                        .push(grammers_session::types::ChannelState { id, pts });
                }
            }

            drop(data);
            self.save()
        })
    }
}

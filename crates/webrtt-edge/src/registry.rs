use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::session::Session;

pub struct SessionRegistry {
    sessions: RwLock<HashMap<Uuid, Arc<RwLock<Session>>>>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn insert(&self, session: Session) -> Arc<RwLock<Session>> {
        let id = session.id;
        let session = Arc::new(RwLock::new(session));
        self.sessions.write().await.insert(id, Arc::clone(&session));
        session
    }

    pub async fn get(&self, id: &Uuid) -> Option<Arc<RwLock<Session>>> {
        self.sessions.read().await.get(id).cloned()
    }

    pub async fn remove(&self, id: &Uuid) {
        self.sessions.write().await.remove(id);
    }

    #[allow(dead_code)]
    pub async fn count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

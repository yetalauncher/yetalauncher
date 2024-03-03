use std::time::Duration;

use log::*;
use tokio::{sync::mpsc::*, time::sleep};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct InternalNotifier {
    receiver: UnboundedReceiver<InternalNotif>,
    sender: UnboundedSender<InternalNotif>,
    notifications: Vec<InternalNotif>
}

#[derive(Debug, Clone)]
pub struct NotifSender {
    inner: UnboundedSender<InternalNotif>,
    id: u32
}

#[derive(Debug, Clone)]
pub struct Notif {
    pub text: String,
    pub progress: u32,
    pub max_progress: u32,
    pub status: NotificationState
}

#[derive(Debug, Clone)]
struct InternalNotif {
    inner: Notif,
    id: u32
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationState {
    Running,
    Success,
    Warning,
    Error
}


impl NotifSender {
    pub fn make_new(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id: fastrand::u32(..)
        }
    }

    fn notify(&self, notif: InternalNotif) {
        self.inner.send(notif).map_err(
            |err| error!("Failed to send notification: {err}")
        ).ok();
    }

    pub fn send(&self, notif: Notif) {
        self.notify(InternalNotif {
            inner: notif,
            id: self.id
        })
    }

    pub fn send_msg(&self, message: &str) {
        self.send(Notif {
            text: message.to_string(),
            ..Default::default()
        })
    }
}

impl InternalNotifier {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            receiver,
            sender,
            notifications: Vec::new()
        }
    }

    pub fn make_notifier(&self) -> NotifSender {
        NotifSender {
            inner: self.sender.clone(),
            id: fastrand::u32(..)
        }
    }

    pub async fn subscribe(&mut self, cancel: CancellationToken, on_update: impl Fn(Vec<&Notif>) -> ()) {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                Some(notif) = self.receiver.recv() => {
                    let exists = self.notifications.iter_mut().find(
                        |other| other.id == notif.id 
                    );

                    let temp_clone = notif.clone();
                    
                    if let Some(existing) = exists {
                        *existing = notif;
                    } else {
                        self.notifications.push(notif);
                    }

                    on_update(self.notifications.iter().map(|n| &n.inner).collect());

                    let timeout = match &temp_clone.inner.status {
                        NotificationState::Success => Some(3),
                        NotificationState::Warning => Some(7),
                        NotificationState::Error => Some(10),
                        _ => None
                    };

                    if let Some(secs) = timeout {
                        sleep(Duration::from_secs(secs)).await;
                        let index = self.notifications.iter().position(
                            |notif| notif.id == temp_clone.id && notif.inner.status == temp_clone.inner.status
                        );

                        if let Some(i) = index {
                            self.notifications.remove(i);
                            
                            on_update(self.notifications.iter().map(|n| &n.inner).collect());
                        }
                    }
                }
            }
        }
    }
}

impl Notif {
    pub fn to_slint(&self) {

    }
}

impl Default for Notif {
    fn default() -> Self {
        Self {
            text: String::new(),
            progress: 0,
            max_progress: 0,
            status: NotificationState::Running
        }
    }
}
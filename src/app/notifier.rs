use std::time::Duration;

use log::*;
use tokio::{sync::mpsc::*, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::slint_generatedMainWindow::{SlNotif, SlNotifState};


#[derive(Debug)]
pub struct InternalNotifier {
    receiver: UnboundedReceiver<InternalNotif>,
    sender: UnboundedSender<InternalNotif>,
    notifications: Vec<InternalNotif>
}

#[derive(Debug, Clone)]
pub struct Notifier {
    inner: UnboundedSender<InternalNotif>,
    id: u32,
    progress: Option<(u32, u32)>
}

#[derive(Debug, Clone)]
pub struct Notif {
    pub text: String,
    pub progress: u32,
    pub max_progress: u32,
    pub status: NotificationState,
    pub in_view: bool
}

#[derive(Debug, Clone)]
struct InternalNotif {
    inner: Notif,
    id: u32,
    typ: InternalNotifType
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationState {
    Running,
    Success,
    Warning,
    Error
}

#[derive(Debug, Clone)]
enum InternalNotifType {
    Schedule,
    FadeIn,
    FadeOut,
    Remove
}


impl Notifier {
    pub fn make_new(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id: fastrand::u32(..),
            progress: None
        }
    }

    pub fn set_progress(&mut self, cur: u32, max: u32) {
        self.progress = Some((cur, max))
    }

    fn notify(&self, notif: InternalNotif) {
        self.inner.send(notif).unwrap_or_else(
            |err| error!("Failed to send notification: {err}")
        );
    }

    pub fn send_remove(&self) {
        self.notify(InternalNotif {
            inner: Notif::default(),
            id: self.id,
            typ: InternalNotifType::Remove
        })
    }

    pub fn send_notif(&self, notif: Notif) {
        self.notify(InternalNotif {
            inner: notif,
            id: self.id,
            typ: InternalNotifType::Schedule
        })
    }

    pub fn send(&self, text: &str, status: NotificationState) {
        self.send_notif(Notif {
            text: text.to_string(),
            progress: if let Some((cur, _)) = self.progress { cur } else { 0 },
            max_progress: if let Some((_, max)) = self.progress { max } else { 0 },
            status,
            ..Default::default()
        })
    }

    pub fn send_msg(&self, message: &str) {
        self.send(message, NotificationState::Running)
    }

    pub fn send_progress(&mut self, message: &str, progress: u32) {
        if let Some((prog, _)) = &mut self.progress { *prog = progress }
        self.send(message, NotificationState::Running)
    }

    pub fn send_success(&self, message: &str) {
        self.send(message, NotificationState::Success)
    }

    pub fn send_error(&self, message: &str) {
        self.send(message, NotificationState::Error)
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

    pub fn make_notifier(&self) -> Notifier {
        Notifier {
            inner: self.sender.clone(),
            id: fastrand::u32(..),
            progress: None
        }
    }

    pub async fn subscribe(&mut self, cancel: CancellationToken, on_update: impl Fn(Vec<&Notif>) -> ()) {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                Some(mut notif) = self.receiver.recv() => match notif.typ {
                    InternalNotifType::Schedule => {
                        let exists = self.notifications.iter_mut().find(
                            |other| other.id == notif.id 
                        );
    
                        let temp_clone = notif.clone();
                        
                        if let Some(existing) = exists {
                            notif.inner.in_view = true;
                            *existing = notif;
                        } else {
                            self.notifications.push(notif);
                        }
    
                        on_update(self.notifications.iter().map(|n| &n.inner).collect());
    
                        let sender = self.sender.clone();

                        tokio::spawn(async move {
                            sleep(Duration::from_millis(10)).await;
                            sender.send(temp_clone.with_typ(InternalNotifType::FadeIn))
                        });
                    },
                    InternalNotifType::FadeIn => {
                        if let Some(existing) = self.notifications.iter_mut().find(|n| n.id == notif.id) {
                            existing.inner.in_view = true;

                            on_update(self.notifications.iter().map(|n| &n.inner).collect());

                            let timeout = match &notif.inner.status {
                                NotificationState::Success => Some(3),
                                NotificationState::Warning => Some(7),
                                NotificationState::Error => Some(10),
                                _ => None
                            };
        
                            if let Some(secs) = timeout {
                                let sender = self.sender.clone();
    
                                tokio::spawn(async move {
                                    sleep(Duration::from_secs(secs)).await;
                                    sender.send(notif.with_typ(InternalNotifType::FadeOut))
                                });
                            }
                        }
                    },
                    InternalNotifType::FadeOut => {
                        if let Some(existing) = self.notifications.iter_mut().find(|n| n.id == notif.id) {
                            existing.inner.in_view = false;

                            on_update(self.notifications.iter().map(|n| &n.inner).collect());

                            let sender = self.sender.clone();
    
                            tokio::spawn(async move {
                                sleep(Duration::from_millis(150)).await;
                                sender.send(notif.with_typ(InternalNotifType::Remove))
                            });
                        }
                    },
                    InternalNotifType::Remove => {
                        let index = self.notifications.iter().position(
                            |other| other.id == notif.id && other.inner.status == notif.inner.status
                        );

                        if let Some(i) = index {
                            self.notifications.remove(i);
                        }

                        on_update(self.notifications.iter().map(|n| &n.inner).collect());
                    }
                }
            }
        }
    }
}

impl InternalNotif {
    fn with_typ(self, typ: InternalNotifType) -> Self {
        Self { typ, ..self }
    }
}

impl Notif {
    pub fn to_slint(&self) -> SlNotif {
        SlNotif {
            text: self.text.to_string().into(),
            progress: (self.progress as i32).into(),
            max_progress: (self.max_progress as i32).into(),
            in_view: self.in_view.clone(),
            status: self.status.to_slint()
        }
    }
}

impl NotificationState {
    pub fn to_slint(&self) -> SlNotifState {
        match self {
            NotificationState::Running => SlNotifState::Running,
            NotificationState::Success => SlNotifState::Success,
            NotificationState::Warning => SlNotifState::Warning,
            NotificationState::Error => SlNotifState::Error
        }
    }
}

impl Default for Notif {
    fn default() -> Self {
        Self {
            text: String::new(),
            progress: 0,
            max_progress: 0,
            in_view: false,
            status: NotificationState::Running
        }
    }
}
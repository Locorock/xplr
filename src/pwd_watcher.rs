use crate::app::Task;
use crate::app::{ExternalMsg, MsgIn};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

pub fn keep_watching(
    pwd: &str,
    tx_msg_in: Sender<Task>,
    rx_pwd_watcher: Receiver<String>,
) -> Result<()> {
    let mut pwd = PathBuf::from(pwd);
    let mut last_modified = pwd.metadata().and_then(|m| m.modified())?;

    thread::spawn(move || loop {
        if let Ok(new_pwd) = rx_pwd_watcher.try_recv() {
            pwd = PathBuf::from(new_pwd);
        } else {
            pwd.metadata()
                .and_then(|m| m.modified())
                .map(|modified| {
                    if modified != last_modified {
                        let msg = MsgIn::External(ExternalMsg::ExplorePwdAsync);
                        tx_msg_in.send(Task::new(msg, None)).unwrap();
                        last_modified = modified;
                    } else {
                        thread::sleep(Duration::from_secs(1));
                    };
                })
                .unwrap_or_else(|e| {
                    let msg = MsgIn::External(ExternalMsg::LogError(e.to_string()));
                    tx_msg_in.send(Task::new(msg, None)).unwrap();
                    thread::sleep(Duration::from_secs(1));
                })
        }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::mpsc;
    #[test]
    fn test_pwd_watcher() {
        let (tx_msg_in, rx_msg_in) = mpsc::channel();
        let (tx_pwd_watcher, rx_pwd_watcher) = mpsc::channel();

        let result = keep_watching("/", tx_msg_in, rx_pwd_watcher);

        assert!(result.is_ok());

        tx_pwd_watcher.send("/bin".to_string()).unwrap();
        let task = rx_msg_in.recv().unwrap();

        let msg = MsgIn::External(ExternalMsg::ExplorePwdAsync);

        assert_eq!(task, Task::new(msg, None));
    }
}

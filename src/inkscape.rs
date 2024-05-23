use crate::{InkscapeArgs, InkscapeArgsBuilder};

use super::AppUI;
use futures::{future::{Fuse, FusedFuture, FutureExt}, pin_mut, select};
use slint::ComponentHandle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub enum InkscapeMessage {
    Quit,
    Export,
}

pub struct InkscapeWorker {
    pub channel: UnboundedSender<InkscapeMessage>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl InkscapeWorker {
    pub fn new(ui: &AppUI) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let handle_weak = ui.as_weak();
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(inkscape_worker_loop(rx, handle_weak))
            }
        });
        Self {
            channel: tx,
            worker_thread,
        }
    }

    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(InkscapeMessage::Quit);
        self.worker_thread.join()
    }
}

async fn inkscape_worker_loop(
    mut rx: UnboundedReceiver<InkscapeMessage>,
    handle: slint::Weak<AppUI>,
) {
// ) -> tokio::io::Result<()> {
    let inkscape_handle = tokio::task::spawn(run_inkscape(handle.clone()));
    // let run_inkscape_future = Fuse::terminated();
    // pin_mut!(run_inkscape_future);
    loop {
        let m = rx.recv().await;
        match m {
            None => return,
            Some(InkscapeMessage::Export) => {
                println!("Exporting...");
                return;
            },
            Some(InkscapeMessage::Quit) => {
                inkscape_handle.abort();
                return;
            },
        }
    }
}

async fn run_inkscape(handle: slint::Weak<AppUI>) {
    let args = InkscapeArgsBuilder::new();
    // let mut cmd = Command::new("inkscape");
    // cmd.arg("--export-type=png");
    // cmd.arg(args.file_path_input.as_ref().unwrap());
    // cmd.arg("--export-filename=test.png");
    // let _ = cmd.spawn();
    // let _ = cmd.wait();
    println!("Executing {}", args.build());
}

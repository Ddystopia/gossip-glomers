#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

pub mod node;

use node::{Node, State};

use std::fmt::Display;
use std::io::{BufRead, Stdin, StdoutLock, Write};
use std::sync::mpsc;
use std::thread;

use anyhow::{bail, Context};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MessageSerde<Payload> {
    pub src: NodeId,
    #[serde(rename = "dest")]
    pub dst: NodeId,
    pub body: BodySerde<Payload>,
}

#[derive(Serialize, Deserialize)]
pub struct BodySerde<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

struct MessageData {
    src: NodeId,
    id: Option<usize>,
    #[allow(dead_code)]
    in_reply_to: Option<usize>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum InitPayload {
    Init(Init),
}

#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitOk {
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(transparent)]
pub struct NodeId(String);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)?;
        Ok(())
    }
}

pub trait Response<OP> {
    type IntoIter: IntoIterator<Item = AtomicResponce<OP>>;
    fn into_iter(self) -> Self::IntoIter;
}

impl<'a, OP: 'a, T: IntoIterator<Item = AtomicResponce<OP>>> Response<OP> for T {
    type IntoIter = Self;
    fn into_iter(self) -> T {
        self
    }
}

pub struct AtomicResponce<OP> {
    recipient: NodeId,
    payload: OP,
    is_reply: bool,
}

impl<OP> AtomicResponce<OP> {
    pub fn new(recipient: NodeId, payload: OP) -> Self {
        Self {
            recipient,
            payload,
            is_reply: true,
        }
    }

    pub fn send(recipient: NodeId, payload: OP) -> Self {
        Self {
            recipient,
            payload,
            is_reply: false,
        }
    }
}

impl<OP> IntoIterator for AtomicResponce<OP> {
    type Item = Self;
    type IntoIter = std::iter::Once<Self>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

pub fn main_loop<'a, N, IP, OP>() -> anyhow::Result<()>
where
    N: Node<IP, OP, StdoutLock<'a>>,
    IP: DeserializeOwned + Send + 'static,
    OP: Serialize,
{
    let stdin = std::io::stdin();
    let stdout = std::io::stdout().lock();

    let init_msg: MessageSerde<InitPayload> = get_init_message(&stdin)?;

    let state = State::from(init_msg, stdout);

    let (tx, rx) = mpsc::channel();
    let stdin_channel_tx = tx.clone();
    let mut node: N = Node::with_initial_state(state, tx);

    let reply = node.get_state_mut().reply(InitOk::InitOk);
    node.get_state_mut().send_to(reply).expect("Failed To Send");

    thread::spawn(move || -> anyhow::Result<()> {
        let stdin = std::io::stdin().lock().lines();
        for line in stdin {
            let line = line.context("Failed to read line from STDIN")?;
            let input: MessageSerde<IP> =
                serde_json::from_str(&line).context("Failed to deserialize message from STDIN")?;

            if stdin_channel_tx.send(input).is_err() {
                bail!("Failed to send input message.");
            }
        }
        Ok(())
    });

    for message in rx {
        let payload = node.get_state_mut().set_message(message);
        let response = node.step(payload);
        node.get_state_mut().send_to(response)?;
    }

    Ok(())
}

fn get_init_message(stdin: &Stdin) -> anyhow::Result<MessageSerde<InitPayload>> {
    let mut init_json = String::with_capacity(50);
    stdin
        .lock()
        .read_line(&mut init_json)
        .context("Failed to read init message from stdin")?;

    let init = serde_json::from_str(&init_json).expect("Failded to deserialize init message.");
    Ok(init)
}

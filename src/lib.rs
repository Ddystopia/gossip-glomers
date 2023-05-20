pub mod node;

use node::*;

use std::io::{BufRead, Stdin, StdoutLock, Write};
use std::sync::mpsc;
use std::thread;

use anyhow::{bail, Context};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
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

pub fn main_loop<'a, N, IP, OP>() -> anyhow::Result<()>
where
    N: Node<IP, OP, StdoutLock<'a>>,
    IP: DeserializeOwned + Send + 'static,
{
    let mut stdin = std::io::stdin();
    let stdout = std::io::stdout().lock();

    let init_msg: Message<InitPayload> = get_init_message(&mut stdin)?;

    let state = State::from(init_msg, stdout);

    let (tx, rx) = mpsc::channel();
    let stdin_channel_tx = tx.clone();
    let mut node: N = Node::with_initial_state(state, tx);

    node.get_state()
        .reply(InitOk::InitOk)
        .context("Initial reply failed")?;

    thread::spawn(move || -> anyhow::Result<()> {
        let stdin = std::io::stdin().lock().lines();
        for line in stdin {
            let line = line.context("Failed to read line from STDIN")?;
            let input: Message<IP> =
                serde_json::from_str(&line).context("Failed to deserialize message from STDIN")?;

            if stdin_channel_tx.send(input).is_err() {
                bail!("Failed to send input message.");
            }
        }
        Ok(())
    });

    for message in rx {
        let payload = node.get_state().set_message(message);
        node.step(payload).context("Node step funciton failed.")?;
    }

    Ok(())
}

fn get_init_message(stdin: &mut Stdin) -> anyhow::Result<Message<InitPayload>> {
    let mut init_json = String::with_capacity(50);
    stdin
        .lock()
        .read_line(&mut init_json)
        .context("Failed to read init message from stdin")?;

    let init = serde_json::from_str(&init_json).expect("Failded to deserialize init message.");
    Ok(init)
}

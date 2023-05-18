use std::io::{BufRead, StdoutLock, Write};

use anyhow::Context;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<S, Payload> {
    fn from_init(init_state: S, init: Init) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn step(&mut self, input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<S, N, Payload>(state: S) -> anyhow::Result<()>
where
    N: Node<S, Payload>,
    Payload: DeserializeOwned,
{
    let stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let mut stdin = stdin.lines();

    let init_msg: Message<InitPayload> = serde_json::from_str(
        &stdin
            .next()
            .expect("Init message should be present.")
            .context("Failed to read init message from stdin")?,
    )
    .expect("Failded to deserialize init message.");

    let InitPayload::Init(init) = init_msg.body.payload else {
        panic!("Unexpected init payload: {:?}", init_msg.body.payload);
    };
    let mut node: N = Node::from_init(state, init).context("Failed to initialize node.")?;

    let init_reply = Message {
        src: init_msg.dst,
        dst: init_msg.src,
        body: Body {
            id: Some(0),
            in_reply_to: init_msg.body.id,
            payload: InitPayload::InitOk,
        },
    };
    serde_json::to_writer(&mut stdout, &init_reply).context("Serialize responce")?;
    stdout.write_all(b"\n").context("Write newline")?;

    for line in stdin {
        let line = line.context("Failed to read line from STDIN")?;
        let input: Message<Payload> =
            serde_json::from_str(&line).context("Failed to deserialize message from STDIN")?;
        node.step(input, &mut stdout)
            .context("Node step funciton failed.")?;
    }

    Ok(())
}

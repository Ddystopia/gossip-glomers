mod node;

use node::*;
use std::io::{BufRead, Lines, StdinLock, StdoutLock, Write};

use anyhow::Context;

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
    IP: DeserializeOwned,
{
    let stdin = std::io::stdin().lock();
    let stdout = std::io::stdout().lock();
    let mut stdin = stdin.lines();

    let init_msg: Message<InitPayload> = get_init_message(&mut stdin)?;

    let state = State::from(init_msg, stdout);

    let mut node: N = Node::with_initial_state(state);

    node.get_state()
        .reply(InitOk::InitOk)
        .context("Initial reply failed")?;

    for line in stdin {
        let line = line.context("Failed to read line from STDIN")?;
        let input: Message<IP> =
            serde_json::from_str(&line).context("Failed to deserialize message from STDIN")?;

        let payload = node.get_state().set_message(input);
        node.step(payload).context("Node step funciton failed.")?;
    }

    Ok(())
}

fn get_init_message(stdin: &mut Lines<StdinLock>) -> anyhow::Result<Message<InitPayload>> {
    let init_json = stdin
        .next()
        .expect("Init message should be present.")
        .context("Failed to read init message from stdin")?;

    let init = serde_json::from_str(&init_json).expect("Failded to deserialize init message.");
    Ok(init)
}

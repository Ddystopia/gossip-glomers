use rustengun::*;

use anyhow::Context;
use std::collections::HashMap;
use std::io::StdoutLock;
use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

pub struct BroadcastNode {
    pub node: String,
    pub id: usize,
    pub messages: Vec<usize>,
}

impl Node<(), Payload> for BroadcastNode {
    fn from_init(_init_state: (), init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(BroadcastNode {
            node: init.node_id,
            id: 1,
            messages: Vec::new(),
        })
    }
    fn step(&mut self, input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));

        let payload = match reply.body.payload {
            Payload::Broadcast { message } => {
                self.messages.push(message);
                Some(Payload::BroadcastOk)
            }
            Payload::Read => Some(Payload::ReadOk {
                messages: self.messages.clone(),
            }),
            Payload::Topology { .. } => Some(Payload::TopologyOk),
            Payload::TopologyOk | Payload::BroadcastOk | Payload::ReadOk { .. } => None,
        };

        if let Some(payload) = payload {
            reply.body.payload = payload;
            serde_json::to_writer(&mut *stdout, &reply).context("Serialize responce")?;
            stdout.write_all(b"\n").context("Write newline")?;
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}

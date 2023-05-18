use rustengun::*;

use anyhow::Context;
use std::io::StdoutLock;
use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

pub struct EchoNode {
    pub id: usize,
}

impl Node<(), Payload> for EchoNode {
    fn from_init(_init_state: (), _init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(EchoNode { id: 1 })
    }

    fn step(&mut self, input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        let body = match input.body.payload {
            Payload::Echo { echo } => {
                let body = Body {
                    id: Some(self.id),
                    in_reply_to: input.body.id,
                    payload: Payload::EchoOk { echo },
                };
                Some(body)
            }
            Payload::EchoOk { .. } => None,
        };

        if let Some(body) = body {
            let reply = Message {
                src: input.dst,
                dst: input.src,
                body,
            };
            serde_json::to_writer(&mut *stdout, &reply).context("Serialize responce")?;
            stdout.write_all(b"\n").context("Write newline")?;
            self.id += 1;
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, EchoNode, _>(())
}
use anyhow::{bail, Context};
use std::io::StdoutLock;
use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

pub struct EchoNode {
    pub id: usize,
}

impl EchoNode {
    pub fn step(&mut self, input: Message, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        let body = match input.body.payload {
            Payload::Init { .. } => {
                let body = Body {
                    id: Some(self.id),
                    in_reply_to: input.body.id,
                    payload: Payload::InitOk,
                };
                Some(body)
            }
            Payload::Echo { echo } => {
                let body = Body {
                    id: Some(self.id),
                    in_reply_to: input.body.id,
                    payload: Payload::EchoOk { echo },
                };
                Some(body)
            }
            Payload::EchoOk { .. } => None,
            Payload::InitOk {} => bail!("Unexpected InitOk"),
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

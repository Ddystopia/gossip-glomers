use rustengun::*;

use anyhow::{bail, Context};
use std::io::StdoutLock;
use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
    Generate,
}

pub struct GenerateNode {
    pub node: String,
    pub id: usize,
}

impl Node<(), Payload> for GenerateNode {
    fn from_init(_init_state: (), init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(GenerateNode {
            node: init.node_id,
            id: 1,
        })
    }
    fn step(&mut self, input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        let body = match input.body.payload {
            Payload::Generate { .. } => {
                let guid = format!("{}-{}", self.node, self.id);
                let body = Body {
                    id: Some(self.id),
                    in_reply_to: input.body.id,
                    payload: Payload::GenerateOk { guid },
                };
                Some(body)
            }
            Payload::GenerateOk { .. } => bail!("Unexpected GenerateOk"),
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
    main_loop::<_, GenerateNode, _>(())
}

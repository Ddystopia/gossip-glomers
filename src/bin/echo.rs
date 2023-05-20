use rustengun::*;

use std::io::StdoutLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum IPayload {
    Echo { echo: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum OPayload {
    EchoOk { echo: String },
}

pub struct EchoNode {
    pub id: usize,
}

impl NodeLogic<IPayload, OPayload> for EchoNode {
    fn from_init(_init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(EchoNode { id: 1 })
    }
fn step(
        &mut self,
        mut input: Message<IPayload>,
        responder: &mut Responder<StdoutLock>,
    ) -> anyhow::Result<()> {
        let payload = match &mut input.body.payload {
            IPayload::Echo { echo } => Some(OPayload::EchoOk {
                echo: std::mem::take(echo),
            }),
        };

        if let Some(payload) = payload {
            responder.reply(input, payload)?;
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<EchoNode, _, _>()
}

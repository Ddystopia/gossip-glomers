use rustengun::*;

use std::io::StdoutLock;

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

    fn step(&mut self, mut input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        let payload = match &mut input.body.payload {
            Payload::Echo { echo } => Some(Payload::EchoOk {
                echo: std::mem::take(echo),
            }),
            Payload::EchoOk { .. } => None,
        };

        if let Some(payload) = payload {
            self.reply(input, stdout, payload)?;
        }

        Ok(())
    }

    fn get_id(&mut self) -> usize {
        let mid = self.id;
        self.id += 1;
        mid
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, EchoNode, _>(())
}

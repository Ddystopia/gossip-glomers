use rustengun::*;

use anyhow::bail;
use std::io::StdoutLock;

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
        let payload = match input.body.payload {
            Payload::Generate { .. } => Payload::GenerateOk {
                guid: format!("{}-{}", self.node, self.id),
            },
            Payload::GenerateOk { .. } => bail!("Unexpected GenerateOk"),
        };

        self.reply(input, stdout, payload)?;

        Ok(())
    }
    fn get_id(&mut self) -> usize {
        let mid = self.id;
        self.id += 1;
        mid
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, GenerateNode, _>(())
}

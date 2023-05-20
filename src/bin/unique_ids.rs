use rustengun::*;

use std::io::StdoutLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum IPayload {
    Generate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum OPayload {
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

pub struct GenerateNode {
    pub name: String,
    pub generated_id_counter: usize,
}

impl NodeLogic<IPayload, OPayload> for GenerateNode {
    fn from_init(init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(GenerateNode {
            name: init.node_id,
            generated_id_counter: 0,
        })
    }

    fn step(
        &mut self,
        input: Message<IPayload>,
        responder: &mut Responder<StdoutLock>,
    ) -> anyhow::Result<()> {
        let payload = match input.body.payload {
            IPayload::Generate { .. } => OPayload::GenerateOk {
                guid: format!("{}-{}", self.name, {
                    self.generated_id_counter += 1;
                    self.generated_id_counter
                }),
            },
        };

        responder.reply(input, payload)?;

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<GenerateNode, _, _>()
}

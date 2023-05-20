use rustengun::{
    node::{Node, State},
    *,
};

use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum IPayload {
    Echo { echo: String },
}

#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum OPayload {
    EchoOk { echo: String },
}

pub struct EchoNode<W> {
    state: State<W>,
}

impl<W> Node<IPayload, OPayload, W> for EchoNode<W>
where
    W: Write,
{
    fn step(&mut self, payload: IPayload) -> anyhow::Result<()> {
        let payload = match payload {
            IPayload::Echo { echo } => OPayload::EchoOk { echo },
        };

        self.state.reply(payload)?;

        Ok(())
    }

    fn with_initial_state(state: State<W>) -> Self {
        EchoNode { state }
    }

    fn get_state(&mut self) -> &mut State<W> {
        &mut self.state
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<EchoNode<_>, _, _>()
}

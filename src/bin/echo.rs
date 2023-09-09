use gossip_glomers::{
    main_loop,
    node::{Node, State},
    MessageSerde,
};

use std::{io::Write, sync::mpsc::Sender};

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
    type Response = gossip_glomers::AtomicResponce<OPayload>;
    fn step(&mut self, payload: IPayload) -> Self::Response {
        let payload = match payload {
            IPayload::Echo { echo } => OPayload::EchoOk { echo },
        };

        self.state.reply(payload)
    }

    fn with_initial_state(state: State<W>, _tx: Sender<MessageSerde<IPayload>>) -> Self {
        Self { state }
    }

    fn get_state(&self) -> &State<W> {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut State<W> {
        &mut self.state
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<EchoNode<_>, _, _>()
}

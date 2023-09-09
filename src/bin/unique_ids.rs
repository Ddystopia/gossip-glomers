use gossip_glomers::{
    main_loop,
    node::{Node, State},
    MessageSerde,
};

use std::{
    io::{StdoutLock, Write},
    sync::mpsc::Sender,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum IPayload {
    Generate,
}

#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum OPayload {
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

pub struct GenerateNode<W> {
    state: State<W>,
    generated_id_counter: usize,
}

impl<W> Node<IPayload, OPayload, W> for GenerateNode<W>
where
    W: Write,
{
    type Response = gossip_glomers::AtomicResponce<OPayload>;
    fn step(&mut self, payload: IPayload) -> Self::Response {
        let payload = match payload {
            IPayload::Generate { .. } => OPayload::GenerateOk {
                guid: format!("{}-{}", self.state.node_id(), {
                    self.generated_id_counter += 1;
                    self.generated_id_counter
                }),
            },
        };

        self.state.reply(payload)
    }
    fn with_initial_state(state: State<W>, _tx: Sender<MessageSerde<IPayload>>) -> Self {
        Self {
            state,
            generated_id_counter: 0,
        }
    }

    fn get_state(&self) -> &State<W> {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut State<W> {
        &mut self.state
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<GenerateNode<StdoutLock>, _, _>()
}

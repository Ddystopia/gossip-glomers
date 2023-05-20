use rustengun::*;

use std::collections::{HashMap, HashSet};
use std::io::StdoutLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum IPayload {
    Broadcast {
        message: usize,
    },
    Read,
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
enum OPayload {
    BroadcastOk,
    ReadOk {
        messages: HashSet<usize>,
        // messages: Vec<usize>,
    },
    TopologyOk,
}

struct BroadcastNode {
    pub name: String,
    pub messages: HashSet<usize>,
    pub not_known_to_neiborgs: HashMap<String, HashSet<usize>>,
    pub neighbors: Vec<String>,
}

impl NodeLogic<IPayload, OPayload> for BroadcastNode {
    fn from_init(init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(BroadcastNode {
            name: init.node_id,
            messages: HashSet::default(),
            not_known_to_neiborgs: init
                .node_ids
                .into_iter()
                .map(|nid| (nid, HashSet::default()))
                .collect(),
            neighbors: Vec::new(),
        })
    }
    fn step(
        &mut self,
        mut input: Message<IPayload>,
        responder: &mut Responder<StdoutLock>,
    ) -> anyhow::Result<()> {
        // TODO: try reply in match arms and remove that mut reference
        let payload = match &mut input.body.payload {
            IPayload::Broadcast { message } => {
                self.messages.insert(*message);
                Some(OPayload::BroadcastOk)
            }
            IPayload::Read => Some(OPayload::ReadOk {
                messages: self.messages.clone(),
            }),
            IPayload::Topology { topology } => {
                if let Some(topology) = std::mem::take(&mut topology.remove(&self.name)) {
                    self.neighbors = topology;
                }
                Some(OPayload::TopologyOk)
            }
        };

        if let Some(payload) = payload {
            responder.reply(input, payload)?;
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<BroadcastNode, _, _>()
}

use rustengun::*;

use std::collections::{HashMap, HashSet};
use std::io::StdoutLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: HashSet<usize>,
        // messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

pub struct BroadcastNode {
    pub node: String,
    pub id: usize,
    pub messages: HashSet<usize>,
    pub known: HashMap<String, HashSet<usize>>,
    pub msg_communicated: HashMap<String, HashSet<usize>>,
    pub neighbors: Vec<String>,
}

impl Node<(), Payload> for BroadcastNode {
    fn from_init(_init_state: (), init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(BroadcastNode {
            node: init.node_id,
            id: 1,
            messages: HashSet::default(),
            known: init
                .node_ids
                .into_iter()
                .map(|nid| (nid, HashSet::default()))
                .collect(),
            msg_communicated: HashMap::default(),
            neighbors: Vec::new(),
        })
    }
    fn step(&mut self, mut input: Message<Payload>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        // TODO: try reply in match arms and remove that mut reference
        let payload = match &mut input.body.payload {
            Payload::Broadcast { message } => {
                self.messages.insert(*message);
                Some(Payload::BroadcastOk)
            }
            Payload::Read => Some(Payload::ReadOk {
                messages: self.messages.clone(),
            }),
            Payload::Topology { topology } => {
                if let Some(topology) = std::mem::take(&mut topology.remove(&self.node)) {
                    self.neighbors = topology;
                }
                Some(Payload::TopologyOk)
            }
            Payload::TopologyOk | Payload::BroadcastOk | Payload::ReadOk { .. } => None,
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
    main_loop::<_, BroadcastNode, _>(())
}

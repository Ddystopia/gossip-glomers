use std::sync::mpsc::Sender;

use crate::{
    AtomicResponce, BodySerde, Context, InitPayload, MessageData, MessageSerde, NodeId, Response,
    Serialize, Write,
};

// This is basically an Inheritance. And `State` is a bad name.
pub struct State<W> {
    node_id: NodeId,
    node_ids: Vec<NodeId>,
    req: MessageData,
    id_counter: usize,
    writer: W,
}

impl<W> State<W>
where
    W: Write,
{
    pub fn node_id(&self) -> NodeId {
        self.node_id.clone()
    }
    pub fn node_ids(&self) -> impl Iterator<Item = &NodeId> {
        self.node_ids.iter()
    }
    pub fn from(req: MessageSerde<InitPayload>, writer: W) -> Self {
        let payload = req.body.payload;
        let InitPayload::Init(payload) = payload;
        let node_ids = payload.node_ids.into_iter().map(NodeId).collect();

        let req = MessageData {
            src: req.src,
            id: None,
            in_reply_to: None,
        };

        Self {
            node_id: NodeId(payload.node_id),
            node_ids,
            req,
            writer,
            id_counter: 0,
        }
    }

    pub fn convert<P>(self, req: MessageSerde<P>) -> (P, Self) {
        let payload = req.body.payload;
        let req = MessageData {
            src: req.src,
            id: None,
            in_reply_to: None,
        };
        let state = Self {
            node_id: self.node_id,
            node_ids: self.node_ids,
            req,
            writer: self.writer,
            id_counter: self.id_counter,
        };
        (payload, state)
    }

    pub fn set_message<P>(&mut self, req: MessageSerde<P>) -> P {
        let payload = req.body.payload;
        let req = MessageData {
            src: req.src,
            id: None,
            in_reply_to: None,
        };
        self.req = req;
        payload
    }

    fn generate_message_id(&mut self) -> usize {
        self.id_counter += 1;
        self.id_counter
    }

    pub fn reply<OP>(&mut self, payload: OP) -> AtomicResponce<OP> {
        AtomicResponce::new(self.req.src.clone(), payload)
    }

    pub fn send_to<OP>(&mut self, response: impl Response<OP>) -> anyhow::Result<()>
    where
        OP: Serialize,
    {
        for AtomicResponce {
            recipient,
            payload,
            is_reply,
        } in response.into_iter()
        {
            let msg = MessageSerde {
                src: self.node_id.clone(),
                dst: recipient,
                body: BodySerde {
                    id: Some(self.generate_message_id()),
                    in_reply_to: is_reply.then(|| self.req.id.unwrap()),
                    payload,
                },
            };

            serde_json::to_writer(&mut self.writer, &msg).context("Serialize responce")?;

            self.writer.write_all(b"\n").context("Write newline")?;
        }
        Ok(())
    }

    pub fn get_sender(&self) -> NodeId {
        self.req.src.clone()
    }
}

pub trait Node<IP, OP, W> {
    type Response: Response<OP>;

    fn step(&mut self, payload: IP) -> Self::Response;

    fn with_initial_state(state: State<W>, tx: Sender<MessageSerde<IP>>) -> Self
    where
        Self: Sized;

    fn get_state(&self) -> &State<W>;
    fn get_state_mut(&mut self) -> &mut State<W>;
}

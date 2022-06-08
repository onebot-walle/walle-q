use ricq::msg::MessageChain;
use ricq::structs::{ForwardMessage, ForwardNode, MessageNode};
use ricq::Client;
use serde::Deserialize;
use walle_core::{resp::RespError, MessageSegment};

use crate::database::WQDatabase;
use crate::error;
use crate::parse::{MsgChainBuilder, RQSendable};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Node {
    pub user_id: String,
    pub time: f64,
    pub user_name: String,
    pub message: MaybeNodes,
}

// bad
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum NodeEnum {
    Node(Node),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MaybeNodes {
    Nodes(Vec<NodeEnum>),
    Standards(Vec<MessageSegment>),
}

impl Node {
    #[async_recursion::async_recursion]
    pub(crate) async fn to_forward_message(
        self,
        target: i64,
        group: bool,
        cli: &Client,
        wqdb: &WQDatabase,
    ) -> Result<ForwardMessage, RespError> {
        Ok(match self.message {
            MaybeNodes::Nodes(nodes) => {
                let mut fwd_nodes = Vec::new();
                for node in nodes {
                    match node {
                        NodeEnum::Node(n) => {
                            let fwd_node = n.to_forward_message(target, group, cli, wqdb).await?;
                            fwd_nodes.push(fwd_node);
                        }
                    }
                }
                ForwardNode {
                    sender_id: self
                        .user_id
                        .parse()
                        .map_err(|_| error::bad_param("user_id"))?,
                    time: (self.time / 1000.0) as i32,
                    sender_name: self.user_name,
                    nodes: fwd_nodes,
                }
                .into()
            }
            MaybeNodes::Standards(message) => {
                let elements: MessageChain = match if group {
                    MsgChainBuilder::group_chain_builder(cli, target, message)
                        .build(wqdb)
                        .await?
                } else {
                    MsgChainBuilder::private_chain_builder(cli, target, message)
                        .build(wqdb)
                        .await?
                }
                .ok_or_else(error::empty_message)?
                {
                    RQSendable::Chain(chain) => chain,
                    RQSendable::Forward(_) => return Err(error::bad_param("node")),
                };
                MessageNode {
                    sender_id: self
                        .user_id
                        .parse()
                        .map_err(|_| error::bad_param("user_id"))?,
                    time: (self.time / 1000.0) as i32,
                    sender_name: self.user_name,
                    elements,
                }
                .into()
            }
        })
    }
}

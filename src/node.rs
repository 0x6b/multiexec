use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy)]
pub enum Node {
    Node1,
    Node2,
    Node3,
    Node4,
}

impl From<Node> for &str {
    fn from(value: Node) -> Self {
        match value {
            Node::Node1 => "node1",
            Node::Node2 => "node2",
            Node::Node3 => "node3",
            Node::Node4 => "node4",
        }
    }
}

impl From<u8> for Node {
    fn from(value: u8) -> Self {
        match value {
            0 => Node::Node1,
            1 => Node::Node2,
            2 => Node::Node3,
            3 => Node::Node4,
            _ => panic!("Invalid node value: {}", value),
        }
    }
}

impl From<Node> for u8 {
    fn from(value: Node) -> Self {
        match value {
            Node::Node1 => 0,
            Node::Node2 => 1,
            Node::Node3 => 2,
            Node::Node4 => 3,
        }
    }
}

impl FromStr for Node {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "node1" | "1" => Ok(Node::Node1),
            "node2" | "2" => Ok(Node::Node2),
            "node3" | "3" => Ok(Node::Node3),
            "node4" | "4" => Ok(Node::Node4),
            _ => Err(format!("Invalid node value: {s}")),
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Node1 => write!(f, "node1"),
            Node::Node2 => write!(f, "node2"),
            Node::Node3 => write!(f, "node3"),
            Node::Node4 => write!(f, "node4"),
        }
    }
}

impl AsRef<str> for Node {
    fn as_ref(&self) -> &str {
        match self {
            Node::Node1 => "node1",
            Node::Node2 => "node2",
            Node::Node3 => "node3",
            Node::Node4 => "node4",
        }
    }
}

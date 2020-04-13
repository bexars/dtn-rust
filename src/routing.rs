use crate::cla::cla_handle::ClaHandle;
use crate::cla::cla_handle::HandleId;
use tokio::sync::mpsc::*;
use std::fmt;

pub mod router;

#[derive(Debug, Clone)]
pub enum RoutingMessage {
    AddRoute(Route),
    DropRoute(Route),
    AddClaHandle(HandleId, Sender<MetaBundle>),
    DropClaHandle(HandleId),
    DataRouterHandle(Sender<MetaBundle>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetaBundle {
    bundle: bp7::bundle::Bundle,
    dest: NodeRoute,
    //arrival
}

#[derive(Debug, Clone, PartialEq)]
pub enum RouteType {
    ConvLayer(HandleId),
    Node(String),      // Recursive lookup another node route
    // Bib(String),       // Bundle in Bundle routing  (mimic GRE)
    // Broadcast,  // For bundles that are broadcast on an interface to any listeners

}

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    dest: NodeRoute,
    nexthop: RouteType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeRoute {
    parts: Vec<String>,
}

impl NodeRoute {
    pub fn contains(&self, rhs: NodeRoute) -> bool {
        if self.parts.len() > rhs.parts.len() { return false; };
        for (s, r) in self.parts.iter().zip(rhs.parts.iter()) {
            if s != r { return false };
        }
        true
    }
}

impl From<&str> for NodeRoute {
    fn from(item: &str) -> Self {
        let parts = if item.len() > 0 {
            item.rsplit(".").map(String::from).collect()
        } else {
            Vec::new()
        };
        Self {
            parts,
        }
    }
}

impl From<bp7::bundle::Bundle> for NodeRoute {
    fn from(bun: bp7::bundle::Bundle) -> Self {
        let node = if let Some(n) = bun.primary.destination.node() {
            n
        } else { "".to_string() };
        return NodeRoute::from(&*node);
    }
}

impl fmt::Display for NodeRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut v = self.parts.clone();
        v.reverse();
        write!(f, "{}", v.join("."))
    }
}



#[cfg(test)]
mod tests {
    use test_case::test_case;

    // Not needed for this example, but useful in general
    use super::*;

    #[test_case( "ip.earth",  "ip.earth" ; "when equal")]
    #[test_case( "",  "ip.earth" ; "when empty")]   
    #[test_case( "ip.earth", "firehouse.bexars.com.ip.earth"; "large depth")]

    //    #[test_case(-2, -4 ; "when both operands are negative")]
    fn test_contains(a: &str, b: &str) {
        let nr_a = NodeRoute::from(a);
        let nr_b = NodeRoute::from(b);
        assert!(nr_a.contains(nr_b));
    }

    #[test_case( "ip.earth","" ; "when empty")]   
    #[test_case( "ship.earth", "air.earth"; "same parent")]

    //    #[test_case(-2, -4 ; "when both operands are negative")]
    fn test_not_contains(a: &str, b: &str) {
        let nr_a = NodeRoute::from(a);
        let nr_b = NodeRoute::from(b);
        assert!(! nr_a.contains(nr_b));
    }


    #[test_case( "ip.earth",  "ip.earth" ; "when equal")]
    #[test_case( "",  "" ; "when empty")]   
    #[test_case( "firehouse.bexars.com.ip.earth", "firehouse.bexars.com.ip.earth"; "large depth")]

    //    #[test_case(-2, -4 ; "when both operands are negative")]
    fn test_roundtrip(a: &str, b: &str) {
        let nr_a = NodeRoute::from(a);
        assert_eq!(nr_a.to_string(), b);
    }


}
use crate::cla::cla_handle::ClaHandle;
use crate::cla::cla_handle::HandleId;
use std::fmt;

pub enum RouteType {
    Loopback,  // Attached to the current node
    IpDns(String),     // Looked up via SRV/A/AAAA records in DNS.  Must start with ip.<geo>  ex. ip.earth
    Ip(String, u16),        // Hardcode DNS or IP address and port
    Node(String),      // Recursive lookup another node route
    Bib(String),       // Bundle in Bundle routing  (mimic GRE)
    Broadcast,  // For bundles that are broadcast on an interface to any listeners

}



pub struct Route {
    dest: NodeRoute,
    nexthop: RouteType,
    interface: Option<HandleId>
}

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
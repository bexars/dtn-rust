use super::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RouteTableEntry {
    pub(super) route: Route,
    pub(super) kids: Vec<RouteTableEntry>,
}

impl RouteTableEntry {  
    // Eventually be smarter about adding entries.  Cloning the whole tree will be painful if this is large
    
    pub fn add(&self, new_route: Route) -> RouteTableEntry {
        let mut new_rte = self.clone();
        new_rte._add(new_route);
        new_rte
    }

    fn _add(&mut self, new_route: Route) {
        let mut new_rte = RouteTableEntry {
            route: new_route.clone(),
            kids: Vec::new(),
        };
        if self.kids.len() > 0 {
            let og_kid_len = self.kids.len();
            for i in 0..self.kids.len() {
                let j = (og_kid_len - 1) -i;
                if  new_route.dest.contains(&self.kids[j].route.dest) {
                    new_rte.kids.push(self.kids.remove(j));
                }
                
            }
        }
        // if !self.route.dest.contains(&route.dest) { return };
        let i = self.kids.iter_mut().filter(|kid| kid.route.dest.contains(&new_route.dest)).next();
        if let Some(kid) = i {
            kid._add(new_route);            
        } else { 
            self.kids.push(new_rte); 
        }
    }

    pub fn lookup(&self, lookup: &NodeRoute) -> Option<HandleId> {
        match self.find(lookup) {
            RouteType::ConvLayer(id) => { return Some(id); },
            _ => { return None },
        }
        
    }

    pub fn find(&self, lookup: &NodeRoute) -> RouteType {
        let i = self.kids.iter().filter(|kid| kid.route.dest.contains(&lookup)).next();
        match i {
            Some(i) => { 
                let res = i.find(lookup);
                match res {
                RouteType::Null => { return i.route.nexthop.clone(); },
                _ => { return res; },
            }}
            None => { return RouteType::Null }
        }
    }

    pub async fn format_routes(&self) -> String {
        fn fmt_parent(parent: &RouteTableEntry, out: &mut String, indent:String) {
            let o = format!( "{}{}   Nexthop:   {}\n", indent, parent.route.dest, parent.route.nexthop);
            out.push_str(&o);
            let indent = format!("{}    ", indent);
            for k in &parent.kids {
                fmt_parent(&k, &mut *out, indent.clone());
            };
        };
        
       
        let mut out = String::new();
        let indent = String::new();
        fmt_parent(&self, &mut out, indent);
        // println!("{}", out);
        out
    }
}

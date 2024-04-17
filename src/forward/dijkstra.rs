// @file      :  dijkstra.rs
// @author    :  fumenglin
// @time      :  2024/4/3 16:25
// @describe  :  下面节点利用迪杰斯特拉算法寻找最短距离


use std::collections::{HashMap};
use crate::common::config::{Point, Node};

#[allow(dead_code)]
pub struct Dijkstra {
    pub graph: HashMap<String, Vec<Point>>,
}

impl Dijkstra {
    #[allow(dead_code)]
    pub fn new(graph: HashMap<String, Vec<Point>>) -> Self {
        Dijkstra {
            graph
        }
    }
    #[allow(dead_code)]
    pub fn create_graph(&mut self, node: Node) {
        if let Some(next) = node.clone().next {
            for nxt in next {
                if let Some(nt) = nxt {
                    //上到下的映射
                    self.update_graph(node.clone(), nt.clone());
                    self.update_graph(nt.clone(), node.clone());
                    self.create_graph(nt);
                }
            }
        }
    }
    #[allow(dead_code)]
    fn update_graph(&mut self, node1: Node, node2: Node) {
        let name = node1.name.clone();
        let  value = self.graph.get(&name);
        if let Some(val) = value {
            val.clone().insert(0, Point::new(node2.name.clone(), node2.addr.clone()));
            self.graph.insert(name, val.clone());
        } else {
            self.graph.insert(name, vec![Point::new(node2.name.clone(), node2.addr.clone())]);
        }
    }

}
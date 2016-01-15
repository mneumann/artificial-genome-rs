#![feature(slice_patterns)]
#![allow(dead_code)]

extern crate artificial_genome;
extern crate rand;

use artificial_genome::{Genome, ProteinRegulator, GeneNetwork, GeneNetworkState};
use artificial_genome::base4::{Base4, B0, B1};
//use std::str::FromStr;
use std::mem;

#[derive(Debug)]
struct Edge {
    src_node: usize,
    dst_node: usize,
    length: f32,
    // Every edge has it's own regulatory network state embedded.
    network_state: GeneNetworkState,
}

impl Edge {
    fn develop(&mut self,
               network: &GeneNetwork,
               next_node_id: &mut usize,
               new_edges: &mut Vec<Edge>) {
        // first perform the state transition.
        // for each node, iterate over all incoming edges. sum

        // create an empty new state.
        let mut new_state = network.new_state();

        for (i, node) in network.nodes().iter().enumerate() {
            // determine the new state of ```node``` (position i in new_state)
            if node.sum_edges(&self.network_state) > 0 {
                // node is enabled
                new_state.state.insert(i);
            }
        }

        println!("previous state: {:?}", self.network_state);
        self.network_state = new_state;
        println!("new state: {:?}", self.network_state);

        // perform the actions of all active nodes in the gene network. then transition the network to
        // the next state.

        // first node is used for differentiation. in a split/duplicate node, this
        // will be set 0 or 1 in the children.
        // starting from second node, the nodes are mapped to graph grammar rules.

        // fixed map from node-id to graph rule. first node is used for differentiation
        // of the edges when a new edge is created.
        //

        if self.network_state.state.contains(1) {
            // GraphGrammar::Split
            let new_node = *next_node_id;
            *next_node_id += 1;

            // differentiate
            let mut child_state = self.network_state.clone();
            child_state.state.set(0, !self.network_state.state.contains(0));

            let new_edge = Edge {
                src_node: new_node,
                dst_node: self.dst_node,
                length: 0.5 * self.length,
                network_state: child_state,
            };
            self.dst_node = new_node;
            self.length *= 0.5;
            new_edges.push(new_edge);
        }

        if self.network_state.state.contains(2) {
            // GraphGrammar::Duplicate

            // differentiate
            let mut child_state = self.network_state.clone();
            child_state.state.set(0, !self.network_state.state.contains(0));

            let new_edge = Edge {
                src_node: self.src_node,
                dst_node: self.dst_node,
                length: self.length,
                network_state: child_state,
            };
            new_edges.push(new_edge);
        }

        if self.network_state.state.contains(3) {
            // GraphGrammar::Swap
            mem::swap(&mut self.dst_node, &mut self.src_node);
        }

        if self.network_state.state.contains(4) {
            // GraphGrammar::Resize grow
            self.length *= 1.25;
        }

        if self.network_state.state.contains(5) {
            // GraphGrammar::Resize shrink
            self.length *= 0.75;
        }
    }
}

#[derive(Debug)]
struct GraphBuilder {
    edges: Vec<Edge>,
    next_node_id: usize,
    network: GeneNetwork,
}

impl GraphBuilder {
    fn new(network: GeneNetwork, zygote: GeneNetworkState) -> GraphBuilder {
        let initial_edge = Edge {
            src_node: 0,
            dst_node: 1,
            length: 1.0,
            network_state: zygote,
        };

        GraphBuilder {
            edges: vec![initial_edge],
            next_node_id: 2,
            network: network,
        }
    }

    // During the process, some edges will be added (split), some others will be modified.
    fn next(&mut self) {
        let mut new_edges = Vec::new();
        for edge in self.edges.iter_mut() {
            edge.develop(&self.network, &mut self.next_node_id, &mut new_edges);
        }
        self.edges.extend(new_edges);
    }
}


fn main() {
    //let genome = Genome::<Base4>::from_str("...11 _0320_23 <0101> T:0311 2...3 _1022_ 133 <0101> \
                                            //W:3213 121...")
    let mut rng = rand::thread_rng();

    let genome = Genome::<Base4>::random(&mut rng, 5*256);

    // let promoter = BaseString::<Base4>::from_str("0101").unwrap();
    let promoter = [B0, B1, B0, B1];

    let genes: Vec<_> = genome.iter_genes(&promoter, 4).collect();
    println!("{:?}", genes);

    let network = genome.construct_network(&promoter,
                                           4,
                                           &|product| {
                                               if product.last() == Some(&B0) {
                                                   // Inhibitor
                                                   ProteinRegulator::inhibit()
                                               } else {
                                                   // Inhibitor
                                                   ProteinRegulator::enhance()
                                               }
                                           }
                                           );

    println!("{:#?}", network);

    let zygote = network.new_state();
    let mut gb = GraphBuilder::new(network, zygote);
    println!("{:#?}", gb);

    gb.next();
    println!("{:#?}", gb);
}

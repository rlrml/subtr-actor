use std::env;

use subtr_actor::stats::analysis_nodes::{
    graph_with_all_analysis_nodes, graph_with_builtin_analysis_nodes,
};

fn main() {
    let names: Vec<_> = env::args().skip(1).collect();
    let mut graph = if names.is_empty() {
        graph_with_all_analysis_nodes()
    } else {
        match graph_with_builtin_analysis_nodes(&names) {
            Ok(graph) => graph,
            Err(error) => {
                eprintln!("{:?}", error);
                std::process::exit(1);
            }
        }
    };
    match graph.render_ascii_dag() {
        Ok(rendered) => println!("{rendered}"),
        Err(error) => {
            eprintln!("{:?}", error);
            std::process::exit(1);
        }
    }
}

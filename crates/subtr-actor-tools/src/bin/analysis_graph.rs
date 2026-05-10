use subtr_actor::build_timeline_graph;

fn main() {
    let mut graph = build_timeline_graph();
    match graph.render_ascii_dag() {
        Ok(rendered) => println!("{rendered}"),
        Err(error) => {
            eprintln!("{error:?}");
            std::process::exit(1);
        }
    }
}

use subtr_actor::build_legacy_timeline_graph;

fn main() {
    let mut graph = build_legacy_timeline_graph();
    let rendered = graph.render_ascii_dag();
    match rendered {
        Ok(rendered) => println!("{rendered}"),
        Err(error) => {
            eprintln!("{error:?}");
            std::process::exit(1);
        }
    }
}

use sim_lib_discrete_graph::alignment_composition_demo;

fn main() {
    let evidence = alignment_composition_demo().expect("verified staged/DTW composition");
    println!(
        "staged states={:?} indices={:?} cost={} cells={} edges={}",
        evidence.staged_states,
        evidence.staged_indices,
        evidence.staged_cost,
        evidence.staged_cells,
        evidence.staged_edges
    );
    println!(
        "alignment score={} steps={} cells={} edges={}",
        evidence.alignment_score,
        evidence.alignment_steps,
        evidence.alignment_cells,
        evidence.alignment_edges
    );
}

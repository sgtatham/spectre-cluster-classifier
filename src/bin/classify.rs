use substitution_tiling_transducers::{
    builtin::spectre,
    combinatorial::CombSystem,
    common::{DisplayViaSystem, System},
};

use spectre_cluster_classifier::{Queryer, Step, report_classification};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum D23Cluster {
    Diabolo,
    Arms2,
    Arms3,
}

fn d23clusters(cs: &CombSystem) -> Vec<(D23Cluster, Vec<Step>)> {
    let mut clusters = Vec::new();

    let psi = cs.tile_by_name("Psi").unwrap();
    let theta = cs.tile_by_name("Theta").unwrap();

    clusters.push((
        D23Cluster::Diabolo,
        vec![
            Step::new(0, Some(psi), None, 9, 10),
            Step::new(0, Some(psi), None, 9, 10),
        ],
    ));

    clusters.push((
        D23Cluster::Arms2,
        vec![
            Step::new(0, None, Some(theta), 9, 0),
            Step::new(1, None, Some(theta), 9, 10),
            Step::new(2, None, Some(theta), 4, 9),
            Step::new(3, None, Some(theta), 0, 11),
            Step::new(4, None, Some(theta), 9, 10),
            Step::new(5, None, Some(theta), 4, 9),
            Step::new(6, None, Some(theta), 4, 3),
            Step::new(7, None, Some(theta), 4, 13),
            Step::new(8, None, Some(theta), 3, 12),
        ],
    ));

    clusters.push((
        D23Cluster::Arms3,
        vec![
            Step::new(0, None, None, 9, 10),
            Step::new(1, None, None, 4, 9),
            Step::new(2, None, None, 4, 3),
            Step::new(3, None, None, 4, 13),
            Step::new(4, None, None, 3, 12),
            Step::new(5, None, None, 4, 3),
            Step::new(6, None, None, 4, 13),
            Step::new(7, None, None, 3, 12),
            Step::new(8, None, None, 9, 0),
            Step::new(9, None, None, 9, 10),
            Step::new(10, None, None, 4, 9),
            Step::new(11, None, None, 1, 8),
            Step::new(12, None, None, 9, 10),
            Step::new(13, None, None, 4, 9),
            Step::new(14, None, None, 13, 12),
            Step::new(15, None, None, 9, 10),
            Step::new(16, None, None, 4, 9),
            Step::new(17, None, None, 4, 3),
            Step::new(18, None, None, 4, 13),
            Step::new(19, None, None, 3, 12),
            Step::new(20, None, None, 4, 9),
            Step::new(21, None, None, 13, 12),
        ],
    ));

    clusters
}

fn main() {
    let sys = spectre();
    let cs = &sys.cs_fine;

    let (ok, answers) =
        report_classification(cs, Queryer::new(cs, &sys.tr, d23clusters(cs)));
    for (answer, prefixes) in answers {
        for prefix in prefixes {
            println!("{:?} {}", answer, prefix.display(cs));
        }
    }
    if !ok {
        std::process::exit(1);
    }
}

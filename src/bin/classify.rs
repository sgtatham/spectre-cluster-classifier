use substitution_tiling_transducers::{
    builtin::spectre,
    combinatorial::CombSystem,
    common::{DisplayViaSystem, System},
};

use spectre_cluster_classifier::{Queryer, Step, report_classification};

// To write a similar program that looks for a different set of
// clusters of tiles, write a different enumeration type here naming
// them, and change the function below to return descriptions of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum D23Cluster {
    Diabolo,
    Arms2,
    Arms3,
}

fn d23clusters(cs: &CombSystem) -> Vec<(D23Cluster, Vec<Step>)> {
    let mut clusters = Vec::new();

    // In some cases we need to enforce that a particular tile is or
    // is not one of these types in the ordinary Spectre system, so we
    // must fetch their numeric ids from the tiling description.
    let psi = cs.tile_by_name("Psi").unwrap();
    let theta = cs.tile_by_name("Theta").unwrap();

    // Each of these clusters.push() statements inserts a description
    // of a cluster of tiles. The description takes the form of a walk
    // around the cluster that visits every tile and returns to the
    // starting point.
    //
    // The main thing in these descriptions is the pair of numbers at
    // the end of each line, which are indices of edges of a Spectre
    // tile, according to the numbering system used by
    // substitution-tiling-transducers: edges 0 and 13 are on opposite
    // sides of the 'head' of the Spectre tile, and go round the short
    // side of the cape followed by the long side, so that edges 9 and
    // 10 are the two halves of the long edge, or the two consecutive
    // edges in the same direction, whichever way you prefer to look
    // at it.
    //
    // Step::new(..., 9, 10) means that we stepped out of edge 9 of
    // the Spectre we were sitting on, and expected to arrive in edge
    // 10 of another Spectre. (Since 9 and 10 are collectively the
    // long edge, that means these two Spectres are back to back along
    // the whole long edge.)
    //
    // The first number in the Step::new call identifies which tile of
    // the cluster we're sitting on before taking the step. One use
    // for this is if it's not possible to walk around the tiling
    // visiting every tile _once_ (i.e. there is no Hamilton cycle).
    // In that situation, when the same tile is revisited, the two
    // steps that leave it should use the same identifying number.
    // Otherwise the system will report an ambiguity: "hey, this
    // Spectre can be both tile 17 and tile 23 of the same cluster!"
    //
    // Here, we're using it for a different purpose. The diabolo is
    // completely symmetric. So we assign both of its tiles the same
    // index, to avoid a spurious ambiguity report ("this tile can be
    // both #0 and #1 of a diabolo").
    //
    // Some(psi) means that we're requiring each of these tiles to be
    // a Psi tile.
    clusters.push((
        D23Cluster::Diabolo,
        vec![
            Step::new(0, Some(psi), None, 9, 10),
            Step::new(0, Some(psi), None, 9, 10),
        ],
    ));

    // The 2-arms has a more interesting walk around a patch of tiles,
    // so the edge numbers all vary. Now the "what tile type is this?"
    // specificaiton is None, meaning "any tile is acceptable". But
    // the _other_ parameter is set to Some(theta). That's the "what
    // kind of tile are we _not_ allowed to be?" parameter. So here
    // we're saying that a 2-arms consists of 9 tiles connected in the
    // pattern specified by these numbers, _none of which is a Theta_.
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

    // The 3-arms doesn't need to specify either what type a tile
    // _is_, or what type it is not. It just specifies the pattern of
    // connections. That's enough.
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

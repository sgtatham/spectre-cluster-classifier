use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Debug;
use substitution_tiling_transducers::{
    address::{
        AddressOverflowError, FiniteTileAddress, TileAddrSymbol, TileAddress,
    },
    combinatorial::{AddrDFAState, CombSystem, TileAddrDFA, Transducer},
    common::TileIndex,
};

/// Deque-based system for generating all the finite prefixes of a
/// tile address in a given system, until each one is long enough to
/// decide something about the family of tiles it describes.
///
/// You initialise it with a [`TileAddrDFA`] for the system you want
/// to analyse. Then you call `next()` to get a tile address. If that
/// address is too short to know the answer to whatever question you
/// have in mind, you push it back on to the queue via the
/// [`PrefixGenState`] you also received from `next()`, and then later
/// you'll see all the one-symbol extensions of the same prefix
/// extended. But if the address was long enough for you to get what
/// you needed, you _don't_ call `push_back()`, and then that prefix
/// is done with. When all prefixes are done, `next()` returns `None`.
pub struct PrefixGen<'a> {
    dfa: &'a TileAddrDFA,
    queue: VecDeque<PrefixGenState>,
}

/// Opaque container describing the state of a [`PrefixGen`] relating
/// to a particular prefix of tile addresses.
pub struct PrefixGenState(Vec<TileAddrSymbol>, AddrDFAState);

impl<'a> PrefixGen<'a> {
    /// Create a new `PrefixGen` based on a [`TileAddrDFA`].
    pub fn new(dfa: &'a TileAddrDFA) -> Self {
        Self {
            dfa,
            queue: VecDeque::from_iter(
                [PrefixGenState(Vec::new(), AddrDFAState::Start)].into_iter(),
            ),
        }
    }

    /// Return the next unprocessed prefix of a tile address, in the
    /// form of a [`FiniteTileAddress`], plus a `[PrefixGenState]` to
    /// pass to `push_back()` if necessary.
    pub fn next(&mut self) -> Option<(FiniteTileAddress, PrefixGenState)> {
        let vst = self.queue.pop_front()?;
        Some((vst.0.clone().into(), vst))
    }

    /// Put a particular tile address back on to the queue, so that in
    /// future all its one-symbol extensions will be returned from
    /// `next()`.
    pub fn push_back(&mut self, vst: PrefixGenState) {
        for (&sym, &dst) in &self.dfa.trans[&vst.1] {
            let mut v2 = vst.0.clone();
            v2.push(sym);
            self.queue.push_back(PrefixGenState(v2, dst));
        }
    }
}

/// Implementation of [`Classifier`] for identifying clusters of tiles
/// in a fixed shape, via tracing a cyclic path through the whole cluster.
///
/// For each cluster of tiles you want to identify, you provide a list
/// of [`Step`] that trace a path through the cluster, visiting every
/// tile _at least_ once, and returning to the starting tile.
/// `Queryer` will then be able to investigate a tile to see whether
/// its pattern of neighbour tiles is consistent with any position
/// within that cluster.
pub struct Queryer<'a, Cluster: Clone + Copy + Debug + Eq + Ord> {
    #[allow(unused)] // this is for printing things in diagnostics
    cs: &'a CombSystem,
    tr: &'a Transducer,

    clusters: Vec<(Cluster, Vec<Step>)>,
}

/// Type describing a single step through one of the cyclic paths used
/// by [`Classifier`].
#[derive(Debug, Clone, Copy)]
pub struct Step {
    /// A value identifying which tile this is in the cluster. You can
    /// set this to whatever you like, but if your path through the
    /// cluster visits the same tile more than once, the two visits to
    /// the same tile should return the same `tileid`.
    pub tileid: usize,

    /// The type of the tile we expect to be sitting on at the start
    /// of this step. If `None`, then any tile type is accepted.
    pub tiletype: Option<TileIndex>,
    /// A tile type that we expect this tile _not_ to be, in case that
    /// is needed to disambiguate the cluster classification.
    pub nontiletype: Option<TileIndex>,
    /// Which edge of the current tile is crossed by the next step.
    pub srcedge: usize,
    /// Which edge of the destination tile we expect to come in
    /// through, after stepping out of edge `srcedge` of the current
    /// tile.
    pub dstedge: usize,
}

impl Step {
    /// Make a new `Step`.
    pub fn new(
        tileid: usize,
        tiletype: Option<TileIndex>,
        nontiletype: Option<TileIndex>,
        srcedge: usize,
        dstedge: usize,
    ) -> Self {
        Self {
            tileid,
            tiletype,
            nontiletype,
            srcedge,
            dstedge,
        }
    }
}

/// Trait describing the parameter type for [`report_classification()`].
pub trait Classifier {
    /// A simple output type that classifies a tile. [`Queryer`], for
    /// example, sets this to be a tuple consisting of a user-provided
    /// enumeration of tile clusters, plus an integer id indicating a
    /// particular tile within a given cluster.
    type Output: Debug + Copy + Ord + Eq;

    /// Classify a tile based on its address, if possible. The inner
    /// [`Result`] can return a [`ClassificationFailure`] error if the
    /// tile has no classification or multiple classifications. The
    /// outer [`Result`] can return [`AddressOverflowError`] if the
    /// input address is too short to find out the classification at
    /// all.
    fn classify(
        &self,
        t: &FiniteTileAddress,
    ) -> Result<
        Result<Self::Output, ClassificationFailure<Self::Output>>,
        AddressOverflowError,
    >;
}

impl<'a, Cluster: Clone + Copy + Debug + Eq + Ord> Queryer<'a, Cluster> {
    fn is_in_cycle_at_pos(
        &self,
        t: &FiniteTileAddress,
        cycle: &[Step],
        pos: usize,
    ) -> Result<bool, AddressOverflowError> {
        let mut t = Cow::Borrowed(t);
        //println!("iicap {pos} {}", t.display(self.cs));
        for i in (pos..cycle.len()).chain(0..pos) {
            let edge = &cycle[i];
            if edge.tiletype.is_some() || edge.nontiletype.is_some() {
                // Check the tile we're sitting on has the right type
                let TileAddrSymbol::Supertile {
                    parent_type: actual_tiletype,
                    subtile: _,
                } = t.get(1)?
                else {
                    panic!("sym #1 isn't a supertile")
                };
                if edge.tiletype.is_some_and(|t| actual_tiletype != t) {
                    //println!(
                    //    "base {i} {} != {}",
                    //    actual_tiletype.display(self.cs),
                    //    expected_tiletype.display(self.cs)
                    //);
                    return Ok(false);
                }
                // Similarly if it doesn't match the excluded type
                if edge.nontiletype.is_some_and(|t| actual_tiletype == t) {
                    return Ok(false);
                }
            }
            // Pass to the next tile in the cycle
            let (tnew, dstedge) = t.neighbour_partial(edge.srcedge, &self.tr);
            let dstedge = dstedge?;
            t = Cow::Owned(tnew);
            // And see if we came in at the right edge
            if dstedge != edge.dstedge {
                //println!(
                //    "edge {i} {} -> {} != {}",
                //    edge.srcedge, dstedge, edge.dstedge
                //);
                return Ok(false);
            }
        }
        // Got all the way round the cycle with no problems!
        Ok(true)
    }

    fn is_in_cycle(
        &self,
        t: &FiniteTileAddress,
        cycle: &[Step],
    ) -> Result<Option<usize>, AddressOverflowError> {
        for pos in 0..cycle.len() {
            if self.is_in_cycle_at_pos(t, cycle, pos)? {
                return Ok(Some(cycle[pos].tileid));
            }
        }
        Ok(None)
    }

    /// Make a new [`Queryer`], based on a tiling substitution system
    /// and its associated transducer.
    ///
    /// `clusters` should be a list of all the tile clusters you want
    /// to identify, each specified as a vector of [`Step`] describing
    /// a path around the cluster, and a value of `Cluster` (typically
    /// an enumeration type) identifying which cluster it is for the
    /// return value.
    pub fn new(
        cs: &'a CombSystem,
        tr: &'a Transducer,
        clusters: Vec<(Cluster, Vec<Step>)>,
    ) -> Self {
        Self { cs, tr, clusters }
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
/// Error type returned from [`Classifier::classify`].
pub enum ClassificationFailure<T> {
    /// Indicates that a tile corresponded to no classification known
    /// to the classifier.
    Unclassifiable,
    /// Indicates that a tile had more than one valid classification.
    /// A vector of them all is returned.
    Ambiguity(Vec<T>),
}

impl<Cluster: Clone + Copy + Debug + Eq + Ord> Classifier
    for Queryer<'_, Cluster>
{
    /// The output type from the `classify` function for `Queryer` is
    /// a tuple giving a cluster id, and a tile id within the cluster
    /// (taken from [`Step::tileid`]).
    type Output = (Cluster, usize);

    fn classify(
        &self,
        t: &FiniteTileAddress,
    ) -> Result<
        Result<Self::Output, ClassificationFailure<Self::Output>>,
        AddressOverflowError,
    > {
        let mut answers = Vec::new();
        for &(cluster, ref cycle) in self.clusters.iter() {
            let answer = self.is_in_cycle(t, cycle)?;
            //println!("iic {cluster:?} {} -> {answer:?}", t.display(self.cs));
            if let Some(tileid) = answer {
                answers.push((cluster, tileid));
            }
        }
        match answers.len() {
            0 => Ok(Err(ClassificationFailure::Unclassifiable)),
            1 => Ok(Ok(answers[0])),
            _ => Ok(Err(ClassificationFailure::Ambiguity(answers))),
        }
    }
}

/// Report the classification of every possible tile address in a
/// substitution system, provided it can always be determined from a
/// finite prefix of the tile's address.
///
/// `cs` describes the tiling system; `classifier` provides a function
/// that classifies a tile based on a finite address.
///
/// The results are given as a vector of elements, each containing a
/// classification or a [`ClassificationFailure`] error, and a list of
/// tile address prefixes corresponding to that classification. The
/// extra `bool` result just indicates success: it is `true` if there
/// were no `Err` in the returned vector.
pub fn report_classification<C: Classifier>(
    cs: &CombSystem,
    classifier: C,
) -> (
    bool,
    Vec<(
        Result<C::Output, ClassificationFailure<C::Output>>,
        Vec<FiniteTileAddress>,
    )>,
) {
    let dfa = cs.tile_addr_dfa();

    let mut success = true;

    // Classify each prefix
    let mut prefixgen = PrefixGen::new(&dfa);
    let mut options_for_prefix = BTreeMap::new();
    while let Some((prefix, qentry)) = prefixgen.next() {
        match classifier.classify(&prefix) {
            Ok(answer) => {
                //println!("{:?} {}", answer, prefix.display(cs));
                if answer.is_err() {
                    success = false;
                }
                for i in 0..=prefix.len() {
                    options_for_prefix
                        .entry(prefix.prefix(i).unwrap())
                        .or_insert_with(BTreeSet::new)
                        .insert(answer.clone());
                }
            }
            Err(..) => {
                //println!("overflow: {}", prefix.display(cs));
                prefixgen.push_back(qentry);
            }
        }
    }

    // Trim unneeded symbols off the end to find the minimal prefixes
    // that determine a unique type
    let mut by_type = BTreeMap::new();
    for (prefix, possibilities) in &options_for_prefix {
        if possibilities.len() == 1
            && prefix.len().checked_sub(1).is_none_or(|i| {
                options_for_prefix[&prefix.prefix(i).unwrap()].len() != 1
            })
        {
            let c = possibilities.iter().next().unwrap().clone();
            by_type
                .entry(c)
                .or_insert_with(BTreeSet::new)
                .insert(prefix);
        }
    }

    // Put the output into a self-contained form
    let vec = by_type
        .into_iter()
        .map(|(k, v)| (k, v.into_iter().cloned().collect()))
        .collect();

    (success, vec)
}

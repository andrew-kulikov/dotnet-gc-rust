//! A graph of logical objects used to model tracing independently of runtime memory.

use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ObjectId(pub u64);

#[derive(Default)]
pub struct Graph {
    objects: HashMap<ObjectId, Vec<ObjectId>>,
    next_id: u64,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn objects(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.objects.keys().copied()
    }

    pub fn references(&self, object: ObjectId) -> Option<&[ObjectId]> {
        self.objects.get(&object).map(Vec::as_slice)
    }

    pub fn add_object(&mut self) -> ObjectId {
        let id = ObjectId(self.next_id);
        self.next_id = self.next_id.checked_add(1).expect("object IDs exhausted");
        self.objects.insert(id, Vec::new());
        id
    }

    pub fn add_edge(&mut self, source: ObjectId, target: ObjectId) -> Result<(), GraphError> {
        if !self.objects.contains_key(&source) {
            return Err(GraphError::MissingSource(source));
        }
        if !self.objects.contains_key(&target) {
            return Err(GraphError::MissingTarget { source, target });
        }

        self.objects.get_mut(&source).unwrap().push(target);
        Ok(())
    }

    /// Borrows the graph exclusively until collection finishes or is abandoned.
    ///
    /// Allocation, mutation, and a second collection cannot overlap this borrow:
    ///
    /// ```compile_fail
    /// use gc_rust::graph::Graph;
    /// let mut graph = Graph::new();
    /// let marking = graph.begin_collection();
    /// graph.add_object();
    /// marking.mark(&[]).unwrap().sweep();
    /// ```
    ///
    /// ```compile_fail
    /// use gc_rust::graph::Graph;
    /// let mut graph = Graph::new();
    /// let root = graph.add_object();
    /// let marked = graph.begin_collection().mark(&[root]).unwrap();
    /// graph.add_edge(root, root).unwrap();
    /// marked.sweep();
    /// ```
    ///
    /// ```compile_fail
    /// use gc_rust::graph::Graph;
    /// let mut graph = Graph::new();
    /// let marking = graph.begin_collection();
    /// graph.begin_collection().mark(&[]).unwrap().sweep();
    /// marking.mark(&[]).unwrap().sweep();
    /// ```
    pub fn begin_collection(&mut self) -> Marking<'_> {
        Marking { graph: self }
    }

    /// Marks completely, then removes unreachable objects. Errors leave the graph unchanged.
    pub fn collect(&mut self, roots: &[ObjectId]) -> Result<CollectionStats, GraphError> {
        Ok(self.begin_collection().mark(roots)?.sweep())
    }
}

/// A collection that has not yet established a complete live set.
///
/// Dropping this phase cancels collection without changing the graph.
/// Sweep is unavailable until marking succeeds:
///
/// ```compile_fail
/// use gc_rust::graph::Graph;
/// Graph::new().begin_collection().sweep();
/// ```
pub struct Marking<'a> {
    graph: &'a mut Graph,
}

impl<'a> Marking<'a> {
    /// Validates and traces without mutation. On error, releases the unchanged graph.
    pub fn mark(self, roots: &[ObjectId]) -> Result<Marked<'a>, GraphError> {
        let result = trace(self.graph, roots)?;
        Ok(Marked {
            graph: self.graph,
            result,
        })
    }
}

/// A complete live set tied to the graph it was traced from.
///
/// Dropping this phase abandons the result without reclaiming anything.
/// Sweeping consumes it, so the same result cannot be applied twice:
///
/// ```compile_fail
/// use gc_rust::graph::Graph;
/// let mut graph = Graph::new();
/// let marked = graph.begin_collection().mark(&[]).unwrap();
/// marked.sweep();
/// marked.sweep();
/// ```
pub struct Marked<'a> {
    graph: &'a mut Graph,
    result: TraceResult,
}

impl Marked<'_> {
    pub fn trace_result(&self) -> &TraceResult {
        &self.result
    }

    pub fn sweep(self) -> CollectionStats {
        let allocated_before = self.graph.objects.len();
        self.graph
            .objects
            .retain(|object, _| self.result.reachable.contains(object));
        let allocated_after = self.graph.objects.len();
        CollectionStats {
            allocated_before,
            live: self.result.reachable.len(),
            reclaimed: allocated_before - allocated_after,
            allocated_after,
        }
    }
}

/// Object counts, not byte counts: before = live + reclaimed, after = live.
#[derive(Debug, Eq, PartialEq)]
pub struct CollectionStats {
    pub allocated_before: usize,
    pub live: usize,
    pub reclaimed: usize,
    pub allocated_after: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum GraphError {
    MissingRoot(ObjectId),
    MissingSource(ObjectId),
    MissingTarget { source: ObjectId, target: ObjectId },
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct TraceStats {
    /// Counts every root entry, including duplicates.
    pub roots_visited: usize,
    pub objects_discovered: usize,
    /// Counts outgoing references of reachable objects, excluding validation.
    pub edges_examined: usize,
    pub max_pending_work: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TraceResult {
    pub reachable: HashSet<ObjectId>,
    pub stats: TraceStats,
}

/// Validates the whole graph and computes reachability without changing it.
///
/// ```
/// use gc_rust::graph::{Graph, GraphError, trace};
///
/// let mut graph = Graph::new();
/// let parent = graph.add_object();
/// let child = graph.add_object();
/// let detached = graph.add_object();
/// graph.add_edge(parent, child)?;
///
/// let result = trace(&graph, &[parent])?;
/// assert!(result.reachable.contains(&child));
/// assert!(!result.reachable.contains(&detached));
/// assert_eq!(graph.objects().count(), 3);
/// # Ok::<(), GraphError>(())
/// ```
pub fn trace(graph: &Graph, roots: &[ObjectId]) -> Result<TraceResult, GraphError> {
    for &root in roots {
        if !graph.objects.contains_key(&root) {
            return Err(GraphError::MissingRoot(root));
        }
    }

    for (&source, targets) in &graph.objects {
        for &target in targets {
            if !graph.objects.contains_key(&target) {
                return Err(GraphError::MissingTarget { source, target });
            }
        }
    }

    let mut reachable = HashSet::new();
    let mut pending = Vec::new();
    let mut stats = TraceStats {
        roots_visited: roots.len(),
        ..TraceStats::default()
    };

    for &root in roots {
        if reachable.insert(root) {
            pending.push(root);
        }
    }
    stats.max_pending_work = pending.len();

    while let Some(object) = pending.pop() {
        let targets = &graph.objects[&object];
        stats.edges_examined += targets.len();

        for &target in targets {
            if reachable.insert(target) {
                pending.push(target);
            }
        }
        stats.max_pending_work = stats.max_pending_work.max(pending.len());
    }

    stats.objects_discovered = reachable.len();
    Ok(TraceResult { reachable, stats })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    fn id(value: u64) -> ObjectId {
        ObjectId(value)
    }

    fn build_graph(entries: &[(u64, &[u64])]) -> Graph {
        let mut graph = Graph::new();
        for &(source, _) in entries {
            assert_eq!(graph.add_object(), id(source));
        }
        for &(source, targets) in entries {
            for &target in targets {
                graph.add_edge(id(source), id(target)).unwrap();
            }
        }
        graph
    }

    #[test]
    fn adds_objects_and_edges() {
        let mut graph = Graph::new();
        let first = graph.add_object();
        let second = graph.add_object();

        assert_ne!(first, second);
        graph.add_edge(first, second).unwrap();
        graph.add_edge(second, second).unwrap();
        assert_eq!(
            trace(&graph, &[first]).unwrap().reachable,
            HashSet::from([first, second])
        );
        assert_eq!(
            graph.add_edge(id(99), first),
            Err(GraphError::MissingSource(id(99)))
        );
        assert_eq!(
            graph.add_edge(first, id(99)),
            Err(GraphError::MissingTarget {
                source: first,
                target: id(99),
            })
        );
    }

    #[test]
    fn traces_cycles_and_disconnected_objects() {
        let graph = build_graph(&[(0, &[1]), (1, &[2]), (2, &[0]), (3, &[3])]);
        let result = trace(&graph, &[id(0)]).unwrap();

        assert_eq!(result.reachable, HashSet::from([id(0), id(1), id(2)]));
        assert_eq!(result.stats.roots_visited, 1);
        assert_eq!(result.stats.objects_discovered, 3);
        assert_eq!(result.stats.edges_examined, 3);
        assert_eq!(result.stats.max_pending_work, 1);
    }

    #[test]
    fn counts_duplicate_roots_and_edges_without_scanning_twice() {
        let graph = build_graph(&[(0, &[0, 1, 1]), (1, &[])]);
        let result = trace(&graph, &[id(0), id(0)]).unwrap();

        assert_eq!(result.reachable, HashSet::from([id(0), id(1)]));
        assert_eq!(result.stats.roots_visited, 2);
        assert_eq!(result.stats.objects_discovered, 2);
        assert_eq!(result.stats.edges_examined, 3);
        assert_eq!(result.stats.max_pending_work, 1);
    }

    #[test]
    fn records_peak_pending_work() {
        let graph = build_graph(&[(0, &[1, 2]), (1, &[]), (2, &[])]);
        let result = trace(&graph, &[id(0)]).unwrap();

        assert_eq!(result.stats.max_pending_work, 2);
    }

    #[test]
    fn handles_empty_graph_and_empty_roots() {
        assert_eq!(
            trace(&Graph::new(), &[]).unwrap().stats,
            TraceStats::default()
        );

        let graph = build_graph(&[(0, &[])]);
        assert!(trace(&graph, &[]).unwrap().reachable.is_empty());
    }

    #[test]
    fn rejects_missing_roots_and_edges_even_when_unreachable() {
        let graph = build_graph(&[(0, &[])]);
        assert_eq!(trace(&graph, &[id(1)]), Err(GraphError::MissingRoot(id(1))));

        let mut graph = build_graph(&[(0, &[]), (1, &[])]);
        graph.objects.get_mut(&id(1)).unwrap().push(id(2));
        assert_eq!(
            trace(&graph, &[id(0)]),
            Err(GraphError::MissingTarget {
                source: id(1),
                target: id(2),
            })
        );
    }

    #[test]
    fn traces_a_deep_chain_without_recursion() {
        const LENGTH: u64 = 100_000;
        let mut graph = Graph::new();
        for _ in 0..LENGTH {
            graph.add_object();
        }
        for index in 0..LENGTH - 1 {
            graph.add_edge(id(index), id(index + 1)).unwrap();
        }

        let result = trace(&graph, &[id(0)]).unwrap();
        assert_eq!(result.reachable.len(), LENGTH as usize);
        assert_eq!(result.stats.edges_examined, LENGTH as usize - 1);
        assert_eq!(result.stats.max_pending_work, 1);
    }

    fn bfs(graph: &Graph, roots: &[ObjectId]) -> HashSet<ObjectId> {
        let mut queue: VecDeque<_> = roots.iter().copied().collect();
        let mut visited = HashSet::new();

        while let Some(object) = queue.pop_front() {
            if visited.insert(object) {
                queue.extend(graph.objects[&object].iter().copied());
            }
        }

        visited
    }

    #[test]
    fn marks_before_sweeping_and_preserves_reachable_cycles() {
        for link_cycles in [false, true] {
            let mut graph = build_graph(&[(0, &[1]), (1, &[0]), (2, &[3]), (3, &[2])]);
            if link_cycles {
                graph.add_edge(id(1), id(2)).unwrap();
            }
            let before = graph.objects.clone();
            let marked = graph.begin_collection().mark(&[id(0), id(0)]).unwrap();
            let expected = if link_cycles {
                HashSet::from([id(0), id(1), id(2), id(3)])
            } else {
                HashSet::from([id(0), id(1)])
            };

            assert_eq!(marked.trace_result().reachable, expected);
            assert_eq!(marked.graph.objects, before);
            assert_eq!(
                marked.sweep(),
                CollectionStats {
                    allocated_before: 4,
                    live: expected.len(),
                    reclaimed: 4 - expected.len(),
                    allocated_after: expected.len(),
                }
            );
            assert_eq!(graph.objects().collect::<HashSet<_>>(), expected);
            for object in expected {
                assert_eq!(graph.references(object), Some(before[&object].as_slice()));
            }
            assert_eq!(graph.add_object(), id(4));
        }
    }

    #[test]
    fn collects_all_dead_and_already_empty_graphs_without_reusing_ids() {
        let mut graph = build_graph(&[(0, &[1]), (1, &[0])]);
        assert_eq!(
            graph.collect(&[]).unwrap(),
            CollectionStats {
                allocated_before: 2,
                live: 0,
                reclaimed: 2,
                allocated_after: 0,
            }
        );
        assert_eq!(
            graph.collect(&[]).unwrap(),
            CollectionStats {
                allocated_before: 0,
                live: 0,
                reclaimed: 0,
                allocated_after: 0,
            }
        );
        assert_eq!(graph.add_object(), id(2));
        assert_eq!(graph.collect(&[id(0)]), Err(GraphError::MissingRoot(id(0))));
        assert!(graph.references(id(2)).is_some());
    }

    #[test]
    fn failed_marking_preserves_the_entire_graph_and_releases_the_borrow() {
        let mut graph = build_graph(&[(0, &[]), (1, &[])]);
        let before = graph.objects.clone();
        assert_eq!(
            graph.collect(&[id(99)]),
            Err(GraphError::MissingRoot(id(99)))
        );
        assert_eq!(graph.objects, before);

        // Inject corruption that the public mutation API normally prevents.
        graph.objects.get_mut(&id(1)).unwrap().push(id(99));
        let before = graph.objects.clone();
        let next_id = graph.next_id;
        for _ in 0..2 {
            assert_eq!(
                graph.collect(&[id(0)]),
                Err(GraphError::MissingTarget {
                    source: id(1),
                    target: id(99),
                })
            );
            assert_eq!(graph.objects, before);
            assert_eq!(graph.next_id, next_id);
        }
        assert_eq!(graph.add_object(), id(next_id));
        graph.add_edge(id(0), id(next_id)).unwrap();
    }

    #[test]
    fn abandoning_either_phase_leaves_the_graph_unchanged() {
        let mut graph = build_graph(&[(0, &[]), (1, &[1])]);
        let before = graph.objects.clone();
        {
            let _marking = graph.begin_collection();
        }
        assert_eq!(graph.objects, before);
        {
            let marked = graph.begin_collection().mark(&[id(0)]).unwrap();
            assert_eq!(marked.trace_result().reachable, HashSet::from([id(0)]));
        }
        assert_eq!(graph.objects, before);
        assert_eq!(graph.add_object(), id(2));
        assert_eq!(graph.collect(&[id(0)]).unwrap().reclaimed, 2);
    }

    #[test]
    fn generated_allocate_link_collect_sequences_match_a_reference_model() {
        for seed in 0..32_u64 {
            let mut state = seed;
            let mut next = || {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                state >> 32
            };
            let mut graph = Graph::new();
            let mut model = HashMap::<u64, Vec<u64>>::new();
            let mut next_id = 0;

            for _ in 0..100 {
                for _ in 0..next() % 5 {
                    assert_eq!(graph.add_object(), id(next_id));
                    model.insert(next_id, Vec::new());
                    next_id += 1;
                }
                let mut objects: Vec<_> = model.keys().copied().collect();
                objects.sort_unstable();
                if !objects.is_empty() {
                    for _ in 0..next() % 8 {
                        let source = objects[next() as usize % objects.len()];
                        let target = objects[next() as usize % objects.len()];
                        graph.add_edge(id(source), id(target)).unwrap();
                        model.get_mut(&source).unwrap().push(target);
                    }
                }
                let roots: Vec<_> = (0..next() % 5)
                    .filter(|_| !objects.is_empty())
                    .map(|_| objects[next() as usize % objects.len()])
                    .collect();

                // Independent oracle: repeated whole-map scans to a fixed point,
                // with no DFS worklist or calls into the production tracer.
                let mut live: HashSet<_> = roots.iter().copied().collect();
                loop {
                    let previous_count = live.len();
                    for (&source, targets) in &model {
                        if live.contains(&source) {
                            live.extend(targets);
                        }
                    }
                    if live.len() == previous_count {
                        break;
                    }
                }
                let before = model.len();
                model.retain(|object, _| live.contains(object));
                let roots: Vec<_> = roots.into_iter().map(id).collect();
                assert_eq!(
                    graph.collect(&roots).unwrap(),
                    CollectionStats {
                        allocated_before: before,
                        live: live.len(),
                        reclaimed: before - live.len(),
                        allocated_after: live.len(),
                    }
                );
                assert_eq!(graph.objects().count(), model.len());
                for (&object, targets) in &model {
                    let expected: Vec<_> = targets.iter().copied().map(id).collect();
                    assert_eq!(graph.references(id(object)), Some(expected.as_slice()));
                }
            }
        }
    }

    #[test]
    fn agrees_with_bfs_on_generated_graphs() {
        let mut state = 1_u64;
        let mut next = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            state >> 32
        };

        for size in 0..32_u64 {
            for _ in 0..20 {
                let mut graph = Graph::new();
                for _ in 0..size {
                    graph.add_object();
                }
                for source in 0..size {
                    for _ in 0..next() % 5 {
                        graph.add_edge(id(source), id(next() % size)).unwrap();
                    }
                }
                let roots: Vec<_> = (0..(next() % 5))
                    .filter(|_| size > 0)
                    .map(|_| id(next() % size))
                    .collect();

                assert_eq!(
                    trace(&graph, &roots).unwrap().reachable,
                    bfs(&graph, &roots)
                );
            }
        }
    }
}

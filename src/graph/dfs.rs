/*!
Depth-first search
*/
use super::ValIdFilter;
use crate::value::Value;

/// A depth-first search of a value's dependencies matching a given filter.
/// Dependencies not matching the filter are ignored *along with all their descendants*.
/// Fallible filters are supported: in this case, the search will halt.
/// May repeat dependencies.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DepDFS<'a, V, F> {
    /// The frontier of this search
    frontier: Vec<&'a V>,
    /// The filter to apply
    filter: F,
}

impl<'a, V, F> DepDFS<'a, V, F> {
    /// Create a new BFS starting at a given frontier
    #[inline]
    pub fn new(frontier: Vec<&'a V>, filter: F) -> DepDFS<'a, V, F> {
        DepDFS { frontier, filter }
    }
    /// Create a new BFS starting at a given value
    #[inline]
    pub fn new_at(start: &'a V, filter: F) -> DepDFS<'a, V, F> {
        Self::new(vec![start], filter)
    }
}

impl<'a, V, F> Iterator for DepDFS<'a, V, F>
where
    V: Value,
    F: ValIdFilter<V>,
{
    type Item = &'a V;
    fn next(&mut self) -> Option<&'a V> {
        let top = self.frontier.pop()?;
        let filter = &mut self.filter;
        self.frontier.extend(
            top.deps()
                .iter()
                .rev()
                .filter_map(|value| filter.filter(value)),
        );
        Some(top)
    }
}

//TODO: potentially more memory-efficient "finger" implementation, but implements another algorithm
/*

/// A depth-first search of a value's dependencies matching a given filter.
/// This filter maps the results, and may morph their dependencies and/or assert a certain value type.
/// Dependencies not matching the filter are ignored *along with all their descendants*.
/// May repeat dependencies.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DepDFS<V, F> {
    /// The frontier of this search
    frontier: Vec<(V, usize)>,
    /// The filter to apply
    filter: F,
}

impl<V, F> DepDFS<V, F> {
    /// Create a new BFS starting at a given position frontier
    #[inline]
    pub fn new(frontier: Vec<(V, usize)>, filter: F) -> DepDFS<V, F> {
        DepDFS { frontier, filter }
    }
    /// Create a new BFS starting at a given value
    #[inline]
    pub fn new_at(start: V, filter: F) -> DepDFS<V, F> {
        Self::new(vec![(start, 0)], filter)
    }
}

impl<V, F> Iterator for DepDFS<V, F>
where
    V: Value,
    F: FnMut(&ValId) -> Option<V>,
{
    type Item = V;
    fn next(&mut self) -> Option<V> {
        loop {
            let mut push_to_top = None;
            {
                let (top, ix) = self.frontier.last_mut()?;
                while *ix < top.no_deps() {
                    *ix += 1;
                    if let Some(dep) = (self.filter)(top.get_dep(*ix - 1)) {
                        push_to_top = Some(dep);
                        break; // Push this to the top of the dependency stack, repeat
                    }
                }
            }
            if let Some(to_push) = push_to_top {
                self.frontier.push((to_push, 0));
                continue;
            } else {
                break;
            }
        }
        self.frontier.pop().map(|(b, _)| b)
    }
}

*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::VisitedFilter;
    use crate::primitive::finite::Finite;
    use crate::value::tuple::Product;
    use crate::value::TypeId;
    use pretty_assertions::assert_eq;

    #[test]
    fn product_dependency_dfs() {
        let f2: TypeId = Finite(2).into();
        let f3: TypeId = Finite(3).into();
        let f4: TypeId = Finite(4).into();
        let f5: TypeId = Finite(5).into();
        let f6: TypeId = Finite(6).into();
        let p234: TypeId = Product::try_new(vec![f2.clone(), f3.clone(), f4.clone()].into())
            .unwrap()
            .into();
        let p3456: TypeId =
            Product::try_new(vec![f3.clone(), f4.clone(), f5.clone(), f6.clone()].into())
                .unwrap()
                .into();
        let p5_234_3456: TypeId =
            Product::try_new(vec![f5.clone(), p234.clone(), p3456.clone()].into())
                .unwrap()
                .into();
        let filter = VisitedFilter::new();
        let dfs = DepDFS::new_at(p5_234_3456.as_val(), filter);
        let deps: Vec<_> = dfs.collect();
        let expected_deps = &[
            p5_234_3456.as_val(),
            f5.as_val(),
            p234.as_val(),
            f2.as_val(),
            f3.as_val(),
            f4.as_val(),
            p3456.as_val(),
            f6.as_val(),
        ];
        assert_eq!(&deps[..], expected_deps);
    }
}

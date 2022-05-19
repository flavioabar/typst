use super::{StyleChain, StyleVec, StyleVecBuilder};

/// A wrapper around a [`StyleVecBuilder`] that allows to collapse items.
pub struct CollapsingBuilder<'a, T> {
    /// The internal builder.
    builder: StyleVecBuilder<'a, T>,
    /// Staged weak and ignorant items that we can't yet commit to the builder.
    /// The option is `Some(_)` for weak items and `None` for ignorant items.
    staged: Vec<(T, StyleChain<'a>, Option<u8>)>,
    /// What the last non-ignorant item was.
    last: Last,
}

/// What the last non-ignorant item was.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Last {
    Weak,
    Destructive,
    Supportive,
}

impl<'a, T> CollapsingBuilder<'a, T> {
    /// Create a new style-vec builder.
    pub fn new() -> Self {
        Self {
            builder: StyleVecBuilder::new(),
            staged: vec![],
            last: Last::Destructive,
        }
    }

    /// Whether the builder is empty.
    pub fn is_empty(&self) -> bool {
        self.builder.is_empty() && self.staged.is_empty()
    }

    /// Can only exist when there is at least one supportive item to its left
    /// and to its right, with no destructive items in between. There may be
    /// ignorant items in between in both directions.
    ///
    /// Between weak items, there may be at least one per layer and among the
    /// candidates the strongest one (smallest `weakness`) wins. When tied,
    /// the one that compares larger through `PartialOrd` wins.
    pub fn weak(&mut self, item: T, styles: StyleChain<'a>, weakness: u8)
    where
        T: PartialOrd,
    {
        if self.last == Last::Destructive {
            return;
        }

        if self.last == Last::Weak {
            if let Some(i) =
                self.staged.iter().position(|(prev_item, _, prev_weakness)| {
                    prev_weakness.map_or(false, |prev_weakness| {
                        weakness < prev_weakness
                            || (weakness == prev_weakness && item > *prev_item)
                    })
                })
            {
                self.staged.remove(i);
            } else {
                return;
            }
        }

        self.staged.push((item, styles, Some(weakness)));
        self.last = Last::Weak;
    }

    /// Forces nearby weak items to collapse.
    pub fn destructive(&mut self, item: T, styles: StyleChain<'a>) {
        self.flush(false);
        self.builder.push(item, styles);
        self.last = Last::Destructive;
    }

    /// Allows nearby weak items to exist.
    pub fn supportive(&mut self, item: T, styles: StyleChain<'a>) {
        self.flush(true);
        self.builder.push(item, styles);
        self.last = Last::Supportive;
    }

    /// Has no influence on other items.
    pub fn ignorant(&mut self, item: T, styles: StyleChain<'a>) {
        self.staged.push((item, styles, None));
    }

    /// Iterate over the contained items.
    pub fn items(&self) -> impl DoubleEndedIterator<Item = &T> {
        self.builder.items().chain(self.staged.iter().map(|(item, ..)| item))
    }

    /// Return the finish style vec and the common prefix chain.
    pub fn finish(mut self) -> (StyleVec<T>, StyleChain<'a>) {
        self.flush(false);
        self.builder.finish()
    }

    /// Push the staged items, filtering out weak items if `supportive` is
    /// false.
    fn flush(&mut self, supportive: bool) {
        for (item, styles, meta) in self.staged.drain(..) {
            if supportive || meta.is_none() {
                self.builder.push(item, styles);
            }
        }
    }
}

impl<'a, T> Default for CollapsingBuilder<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::layout::FlowChild;
    use crate::library::prelude::*;

    #[track_caller]
    fn test<T>(builder: CollapsingBuilder<T>, expected: &[T])
    where
        T: Debug + PartialEq,
    {
        let result = builder.finish().0;
        let items: Vec<_> = result.items().collect();
        let expected: Vec<_> = expected.iter().collect();
        assert_eq!(items, expected);
    }

    fn node() -> FlowChild {
        FlowChild::Node(Content::Text("Hi".into()).pack())
    }

    fn abs(pt: f64) -> FlowChild {
        FlowChild::Spacing(Length::pt(pt).into())
    }

    #[test]
    fn test_collapsing_weak() {
        let mut builder = CollapsingBuilder::new();
        let styles = StyleChain::default();
        builder.weak(FlowChild::Colbreak, styles, 0);
        builder.supportive(node(), styles);
        builder.weak(abs(10.0), styles, 0);
        builder.ignorant(FlowChild::Colbreak, styles);
        builder.weak(abs(20.0), styles, 0);
        builder.supportive(node(), styles);
        builder.weak(abs(10.0), styles, 0);
        builder.weak(abs(20.0), styles, 1);
        builder.supportive(node(), styles);
        test(builder, &[
            node(),
            FlowChild::Colbreak,
            abs(20.0),
            node(),
            abs(10.0),
            node(),
        ]);
    }

    #[test]
    fn test_collapsing_destructive() {
        let mut builder = CollapsingBuilder::new();
        let styles = StyleChain::default();
        builder.supportive(node(), styles);
        builder.weak(abs(10.0), styles, 0);
        builder.destructive(FlowChild::Colbreak, styles);
        builder.weak(abs(20.0), styles, 0);
        builder.supportive(node(), styles);
        test(builder, &[node(), FlowChild::Colbreak, node()]);
    }
}
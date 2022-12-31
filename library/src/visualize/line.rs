use crate::prelude::*;

/// # Line
/// A line from one point to another.
///
/// ## Example
/// ```
/// #set page(height: 100pt)
/// #line(end: (50%, 50%))
/// ```
///
/// ## Parameters
/// - start: Axes<Rel<Length>> (named)
///   The start point of the line.
///   Must be an array of exactly two relative lengths.
///
/// - end: Axes<Rel<Length>> (named)
///   The end point of the line.
///   Must be an array of exactly two relative lengths.
///
/// - length: Rel<Length> (named)
///   The line's length. Mutually exclusive with `end`.
///
/// - angle: Angle (named)
///   The angle at which the line points away from the origin. Mutually
///   exclusive with `end`.
///
/// ## Category
/// visualize
#[func]
#[capable(Layout, Inline)]
#[derive(Debug, Hash)]
pub struct LineNode {
    /// Where the line starts.
    pub start: Axes<Rel<Length>>,
    /// The offset from `start` where the line ends.
    pub delta: Axes<Rel<Length>>,
}

#[node]
impl LineNode {
    /// How to stroke the line. This can be:
    ///
    /// - A length specifying the stroke's thickness. The color is inherited,
    ///   defaulting to black.
    /// - A color to use for the stroke. The thickness is inherited, defaulting
    ///   to `{1pt}`.
    /// - A stroke combined from color and thickness using the `+` operator as
    ///   in `{2pt + red}`.
    ///
    /// # Example
    /// ```
    /// #line(length: 100%, stroke: 2pt + red)
    /// ```
    #[property(resolve, fold)]
    pub const STROKE: PartialStroke = PartialStroke::default();

    fn construct(_: &Vm, args: &mut Args) -> SourceResult<Content> {
        let start = args.named("start")?.unwrap_or_default();

        let delta = match args.named::<Axes<Rel<Length>>>("end")? {
            Some(end) => end.zip(start).map(|(to, from)| to - from),
            None => {
                let length =
                    args.named::<Rel<Length>>("length")?.unwrap_or(Abs::cm(1.0).into());

                let angle = args.named::<Angle>("angle")?.unwrap_or_default();
                let x = angle.cos() * length;
                let y = angle.sin() * length;

                Axes::new(x, y)
            }
        };

        Ok(Self { start, delta }.pack())
    }
}

impl Layout for LineNode {
    fn layout(
        &self,
        _: &mut Vt,
        styles: StyleChain,
        regions: Regions,
    ) -> SourceResult<Fragment> {
        let stroke = styles.get(Self::STROKE).unwrap_or_default();

        let origin = self
            .start
            .resolve(styles)
            .zip(regions.base)
            .map(|(l, b)| l.relative_to(b));

        let delta = self
            .delta
            .resolve(styles)
            .zip(regions.base)
            .map(|(l, b)| l.relative_to(b));

        let target = regions.expand.select(regions.first, Size::zero());

        let mut frame = Frame::new(target);
        let shape = Geometry::Line(delta.to_point()).stroked(stroke);
        frame.push(origin.to_point(), Element::Shape(shape));

        Ok(Fragment::frame(frame))
    }
}

impl Inline for LineNode {}
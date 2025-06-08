//! Combinators that allow combining and extending existing parsers.
//!
//! *“Ford... you're turning into a penguin. Stop it.”*
//!
//! Although it's *sometimes* useful to be able to name their type, most of these parsers are much easier to work with
//! when accessed through their respective methods on [`Parser`].

use inspector::Inspector;

use super::*;

/// The type of a lazy parser.
pub type Lazy<'src, A, I, E> =
    ThenIgnore<A, Repeated<Any<I, E>, I, E, <I as Input<'src>>::Token>, E, <A as UnitParser<'src, I, E>>::Output>;

/// Alter the configuration of a struct using parse-time context
pub struct Configure<A, F, O> {
    pub(crate) parser: A,
    pub(crate) cfg: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, F: Copy, O> Copy for Configure<A, F, O> {}
impl<A: Clone, F: Clone, O> Clone for Configure<A, F, O> {
    fn clone(&self) -> Self {
        Configure {
            parser: self.parser.clone(),
            cfg: self.cfg.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, F, O> UnitParser<'src, I, E> for Configure<A, F, O>
where
    A: ConfigParser<'src, I, E, Output = O>,
    F: Fn(A::Config, &E::Context) -> A::Config,
    I: Input<'src>,
    E: ParserExtra<'src, I>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
    where
        Self: Sized,
    {
        let cfg = (self.cfg)(A::Config::default(), inp.ctx());
        self.parser.go_cfg::<M>(inp, cfg)
    }

    go_extra!(Self::Output);
}

/// See [`ConfigIterParser::configure`]
pub struct IterConfigure<A, F, OI> {
    pub(crate) parser: A,
    pub(crate) cfg: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<OI>,
}

impl<A: Copy, F: Copy, OI> Copy for IterConfigure<A, F, OI> {}
impl<A: Clone, F: Clone, OI> Clone for IterConfigure<A, F, OI> {
    fn clone(&self) -> Self {
        IterConfigure {
            parser: self.parser.clone(),
            cfg: self.cfg.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, F, OI> UnitParser<'src, I, E> for IterConfigure<A, F, OI>
where
    A: ConfigIterParser<'src, I, E, Item = OI>,
    F: Fn(A::Config, &E::Context) -> A::Config,
    I: Input<'src>,
    E: ParserExtra<'src, I>,
{
    type Output = ();

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, ()> {
        let mut state = self.make_iter::<Check>(inp)?;
        loop {
            match self.next::<Check>(inp, &mut state) {
                Ok(Some(())) => {}
                Ok(None) => break Ok(M::bind(|| ())),
                Err(()) => break Err(()),
            }
        }
    }

    go_extra!(Self::Output);
}

impl<'src, I, E, A, F, OI> IterParser<'src, I, E> for IterConfigure<A, F, OI>
where
    A: ConfigIterParser<'src, I, E, Item = OI>,
    F: Fn(A::Config, &E::Context) -> A::Config,
    I: Input<'src>,
    E: ParserExtra<'src, I>,
{
    type Item = OI;
    type IterState<M: Mode>
        = (A::IterState<M>, A::Config)
    where
        I: 'src;

    const NONCONSUMPTION_IS_OK: bool = A::NONCONSUMPTION_IS_OK;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        Ok((
            A::make_iter(&self.parser, inp)?,
            (self.cfg)(A::Config::default(), inp.ctx()),
        ))
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        state: &mut Self::IterState<M>,
    ) -> IPResult<M, Self::Item> {
        self.parser.next_cfg(inp, &mut state.0, &state.1)
    }
}

/// See [`ConfigIterParser::try_configure`]
pub struct TryIterConfigure<A, F, OI> {
    pub(crate) parser: A,
    pub(crate) cfg: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<OI>,
}

impl<A: Copy, F: Copy, OI> Copy for TryIterConfigure<A, F, OI> {}
impl<A: Clone, F: Clone, OI> Clone for TryIterConfigure<A, F, OI> {
    fn clone(&self) -> Self {
        TryIterConfigure {
            parser: self.parser.clone(),
            cfg: self.cfg.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, F, OI> UnitParser<'src, I, E> for TryIterConfigure<A, F, OI>
where
    A: ConfigIterParser<'src, I, E, Item = OI>,
    F: Fn(A::Config, &E::Context, I::Span) -> Result<A::Config, E::Error>,
    I: Input<'src>,
    E: ParserExtra<'src, I>,
{
    type Output = ();

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, ()> {
        let mut state = self.make_iter::<Check>(inp)?;
        loop {
            match self.next::<Check>(inp, &mut state) {
                Ok(Some(())) => {}
                Ok(None) => break Ok(M::bind(|| ())),
                Err(()) => break Err(()),
            }
        }
    }

    go_extra!(Self::Output);
}

impl<'src, I, E, A, F, OI> IterParser<'src, I, E> for TryIterConfigure<A, F, OI>
where
    A: ConfigIterParser<'src, I, E, Item = OI>,
    F: Fn(A::Config, &E::Context, I::Span) -> Result<A::Config, E::Error>,
    I: Input<'src>,
    E: ParserExtra<'src, I>,
{
    type Item = OI;

    type IterState<M: Mode>
        = (A::IterState<M>, A::Config)
    where
        I: 'src;

    const NONCONSUMPTION_IS_OK: bool = A::NONCONSUMPTION_IS_OK;

    fn make_iter<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        let span = inp.span_since(&inp.cursor());
        let cfg = (self.cfg)(A::Config::default(), inp.ctx(), span)
            .map_err(|e| inp.add_alt_err(&inp.cursor().inner, e))?;

        Ok((A::make_iter(&self.parser, inp)?, cfg))
    }

    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        state: &mut Self::IterState<M>,
    ) -> IPResult<M, Self::Item> {
        self.parser.next_cfg(inp, &mut state.0, &state.1)
    }
}

/// See [`UnitParser::to_slice`]
pub struct ToSlice<A> {
    pub(crate) parser: A,
}

impl<A: Copy> Copy for ToSlice<A> {}
impl<A: Clone> Clone for ToSlice<A> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
        }
    }
}

impl<'src, I, E, A> UnitParser<'src, I, E> for ToSlice<A>
where
    A: UnitParser<'src, I, E>,
    I: SliceInput<'src>,
    E: ParserExtra<'src, I>,
{
    type Output = I::Slice;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, I::Slice>
    where
        Self: Sized,
    {
        let before = inp.cursor();
        self.parser.go::<Check>(inp)?;

        Ok(M::bind(|| inp.slice_since(&before..)))
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::filter`].
pub struct Filter<A, F, O> {
    pub(crate) parser: A,
    pub(crate) filter: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, F: Copy, O> Copy for Filter<A, F, O> {}
impl<A: Clone, F: Clone, O> Clone for Filter<A, F, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            filter: self.filter.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, F, O> UnitParser<'src, I, E> for Filter<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
    F: Fn(&A::Output) -> bool,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let before = inp.cursor();
        self.parser.go::<Emit>(inp).and_then(|out| {
            if (self.filter)(&out) {
                Ok(M::bind(|| out))
            } else {
                let err_span = inp.span_since(&before);
                inp.add_alt([DefaultExpected::SomethingElse], None, err_span);
                Err(())
            }
        })
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::map`].
pub struct Map<A, F, O> {
    pub(crate) parser: A,
    pub(crate) mapper: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, O, F: Copy> Copy for Map<A, F, O> {}
impl<A: Clone, O, F: Clone> Clone for Map<A, F, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            mapper: self.mapper.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, F, O> UnitParser<'src, I, E> for Map<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    F: Fn(A::Output) -> O,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let out = self.parser.go::<M>(inp)?;
        Ok(M::map(out, &self.mapper))
    }

    go_extra!(Self::Output);
}

impl<'src, I, E, A, O, F> IterParser<'src, I, E> for Map<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: IterParser<'src, I, E>,
    F: Fn(A::Item) -> O,
{
    type Item = O;
    type IterState<M: Mode>
        = A::IterState<M>
    where
        I: 'src;

    const NONCONSUMPTION_IS_OK: bool = A::NONCONSUMPTION_IS_OK;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        self.parser.make_iter(inp)
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        state: &mut Self::IterState<M>,
    ) -> IPResult<M, O> {
        match self.parser.next::<M>(inp, state) {
            Ok(Some(o)) => Ok(Some(M::map(o, &self.mapper))),
            Ok(None) => Ok(None),
            Err(()) => Err(()),
        }
    }
}

/// See [`UnitParser::map_with`].
pub struct MapWith<A, F, O> {
    pub(crate) parser: A,
    pub(crate) mapper: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, O, F: Copy> Copy for MapWith<A, F, O> {}
impl<A: Clone, O, F: Clone> Clone for MapWith<A, F, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            mapper: self.mapper.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, F, O> UnitParser<'src, I, E> for MapWith<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    F: Fn(A::Output, &mut MapExtra<'src, '_, I, E>) -> O,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let before = inp.cursor();
        let out = self.parser.go::<M>(inp)?;
        Ok(M::map(out, |out| {
            (self.mapper)(out, &mut MapExtra::new(&before, inp))
        }))
    }

    go_extra!(Self::Output);
}

impl<'src, I, E, A, F, O> IterParser<'src, I, E> for MapWith<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: IterParser<'src, I, E>,
    F: Fn(A::Item, &mut MapExtra<'src, '_, I, E>) -> O,
{
    type Item = O;
    type IterState<M: Mode>
        = A::IterState<M>
    where
        I: 'src;

    const NONCONSUMPTION_IS_OK: bool = A::NONCONSUMPTION_IS_OK;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        self.parser.make_iter(inp)
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        state: &mut Self::IterState<M>,
    ) -> IPResult<M, O> {
        let before = inp.cursor();
        match self.parser.next::<M>(inp, state) {
            Ok(Some(o)) => Ok(Some(M::map(o, |o| {
                (self.mapper)(o, &mut MapExtra::new(&before, inp))
            }))),
            Ok(None) => Ok(None),
            Err(()) => Err(()),
        }
    }
}

/// See [`UnitParser::map_group`].
#[cfg(feature = "nightly")]
pub struct MapGroup<A, F, O> {
    pub(crate) parser: A,
    pub(crate) mapper: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

#[cfg(feature = "nightly")]
impl<A: Copy, O, F: Copy> Copy for MapGroup<A, F, O> {}
#[cfg(feature = "nightly")]
impl<A: Clone, O, F: Clone> Clone for MapGroup<A, F, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            mapper: self.mapper.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

#[cfg(feature = "nightly")]
impl<'src, I, E, A, F, O> UnitParser<'src, I, E> for MapGroup<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    F: Fn<A::Output, Output = O>,
    A::Output: Tuple,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let out = self.parser.go::<M>(inp)?;
        Ok(M::map(out, |out| self.mapper.call(out)))
    }

    go_extra!(Self::Output);
}

#[cfg(feature = "nightly")]
impl<'src, I, E, A, F, O> IterParser<'src, I, E> for MapGroup<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: IterParser<'src, I, E>,
    F: Fn<A::Item, Output = O>,
    A::Item: Tuple,
{
    type Item = O;

    type IterState<M: Mode>
        = A::IterState<M>
    where
        I: 'src;

    const NONCONSUMPTION_IS_OK: bool = A::NONCONSUMPTION_IS_OK;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        self.parser.make_iter(inp)
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        state: &mut Self::IterState<M>,
    ) -> IPResult<M, O> {
        match self.parser.next::<M>(inp, state) {
            Ok(Some(o)) => Ok(Some(M::map(o, |o| self.mapper.call(o)))),
            Ok(None) => Ok(None),
            Err(()) => Err(()),
        }
    }
}

/// See [`UnitParser::to_span`].
pub struct ToSpan<A> {
    pub(crate) parser: A,
}

impl<A: Copy> Copy for ToSpan<A> {}
impl<A: Clone> Clone for ToSpan<A> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
        }
    }
}

impl<'src, I, E, A> UnitParser<'src, I, E> for ToSpan<A>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
{
    type Output = I::Span;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, I::Span> {
        let before = inp.cursor();
        self.parser.go::<M>(inp)?;
        Ok(M::bind(|| inp.span_since(&before)))
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::try_foldl`].
pub struct TryFoldl<F, A, B, E, O> {
    pub(crate) parser_a: A,
    pub(crate) parser_b: B,
    pub(crate) folder: F,
    #[cfg(debug_assertions)]
    pub(crate) location: Location<'static>,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(E, O)>,
}

impl<F: Copy, A: Copy, B: Copy, E, O> Copy for TryFoldl<F, A, B, E, O> {}
impl<F: Clone, A: Clone, B: Clone, E, O> Clone for TryFoldl<F, A, B, E, O> {
    fn clone(&self) -> Self {
        Self {
            parser_a: self.parser_a.clone(),
            parser_b: self.parser_b.clone(),
            folder: self.folder.clone(),
            #[cfg(debug_assertions)]
            location: self.location,
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, F, A, B, E, O> UnitParser<'src, I, E> for TryFoldl<F, A, B, E, O>
where
    I: Input<'src>,
    A: UnitParser<'src, I, E, Output = O>,
    B: IterParser<'src, I, E>,
    E: ParserExtra<'src, I>,
    F: Fn(O, B::Item, &mut MapExtra<'src, '_, I, E>) -> Result<O, E::Error>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
    where
        Self: Sized,
    {
        let before_all = inp.cursor();
        let mut out = self.parser_a.go::<Emit>(inp)?;
        let mut iter_state = self.parser_b.make_iter::<Emit>(inp)?;
        loop {
            let before = inp.cursor();
            match self.parser_b.next::<Emit>(inp, &mut iter_state) {
                Ok(Some(b_out)) => {
                    match (self.folder)(out, b_out, &mut MapExtra::new(&before_all, inp)) {
                        Ok(b_f_out) => {
                            out = b_f_out;
                        }
                        Err(err) => {
                            inp.add_alt_err(&before.inner, err);
                            break Err(());
                        }
                    }
                }
                Ok(None) => break Ok(M::bind(|| out)),
                Err(()) => break Err(()),
            }
            #[cfg(debug_assertions)]
            if !B::NONCONSUMPTION_IS_OK {
                debug_assert!(
                    before != inp.cursor(),
                    "found Foldl combinator making no progress at {}",
                    self.location,
                );
            }
        }
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::try_map`].
pub struct TryMap<A, F, O> {
    pub(crate) parser: A,
    pub(crate) mapper: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, O, F: Copy> Copy for TryMap<A, F, O> {}
impl<A: Clone, O, F: Clone> Clone for TryMap<A, F, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            mapper: self.mapper.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, F, O> UnitParser<'src, I, E> for TryMap<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    F: Fn(A::Output, I::Span) -> Result<O, E::Error>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let before = inp.cursor();
        // Remove the pre-inner alt, to be reinserted later so we always preserve it
        let old_alt = inp.errors.alt.take();

        let out = self.parser.go::<Emit>(inp)?;
        let span = inp.span_since(&before);
        let new_alt = inp.errors.alt.take();

        match (self.mapper)(out, span) {
            Ok(out) => {
                // If successful, reinsert the original alt and then apply the new alt on top of it, since both are valid
                inp.errors.alt = old_alt;
                if let Some(new_alt) = new_alt {
                    inp.add_alt_err(&before.inner, new_alt.err);
                }
                Ok(M::bind(|| out))
            }
            Err(err) => {
                // If unsuccessful, reinsert the original alt but replace the new alt with the mapper error (since it overrides it)
                inp.errors.alt = old_alt;
                inp.add_alt_err(&before.inner, err);
                Err(())
            }
        }
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::try_map_with`].
pub struct TryMapWith<A, F, O> {
    pub(crate) parser: A,
    pub(crate) mapper: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, O, F: Copy> Copy for TryMapWith<A, F, O> {}
impl<A: Clone, O, F: Clone> Clone for TryMapWith<A, F, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            mapper: self.mapper.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, F, O> UnitParser<'src, I, E> for TryMapWith<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    F: Fn(A::Output, &mut MapExtra<'src, '_, I, E>) -> Result<O, E::Error>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let before = inp.cursor();
        let out = self.parser.go::<Emit>(inp)?;
        match (self.mapper)(out, &mut MapExtra::new(&before, inp)) {
            Ok(out) => Ok(M::bind(|| out)),
            Err(err) => {
                inp.add_alt_err(&inp.cursor().inner, err);
                Err(())
            }
        }
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::to`].
pub struct To<A, O> {
    pub(crate) parser: A,
    pub(crate) to: O,
}

impl<A: Copy, O: Copy> Copy for To<A, O> {}
impl<A: Clone, O: Clone> Clone for To<A, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            to: self.to.clone(),
        }
    }
}

impl<'src, I, E, A, O> UnitParser<'src, I, E> for To<A, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    O: Clone,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        self.parser.go::<Check>(inp)?;
        Ok(M::bind(|| self.to.clone()))
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::into_iter`].
pub struct IntoIter<A, O> {
    pub(crate) parser: A,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, O> Copy for IntoIter<A, O> {}
impl<A: Clone, O> Clone for IntoIter<A, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, O> UnitParser<'src, I, E> for IntoIter<A, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    A::Output: IntoIterator<Item = O>,
{
    type Output = ();

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, ()> {
        self.parser.go::<Check>(inp)?;
        Ok(M::bind(|| ()))
    }

    go_extra!(Self::Output);
}

impl<'src, I, E, A, O> IterParser<'src, I, E> for IntoIter<A, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    A::Output: IntoIterator<Item = O>,
{
    type Item = O;

    // TODO: Don't always produce output for non-emitting modes, but needed due to length. Use some way to 'select'
    // between iterator and usize at compile time.
    type IterState<M: Mode> = <<A as UnitParser<'src, I, E>>::Output as IntoIterator>::IntoIter; //M::Output<O::IntoIter>;

    const NONCONSUMPTION_IS_OK: bool = true;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        // M::map(self.parser.go::<M>(inp)?, |out| out.into_iter())
        self.parser.go::<Emit>(inp).map(|out| out.into_iter())
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        _inp: &mut InputRef<'src, '_, I, E>,
        iter: &mut Self::IterState<M>,
    ) -> IPResult<M, Self::Item> {
        Ok(iter.next().map(|out| M::bind(|| out)))
    }
}

/// See [`UnitParser::ignored`].
pub struct Ignored<A> {
    pub(crate) parser: A,
}

impl<A: Copy> Copy for Ignored<A> {}
impl<A: Clone> Clone for Ignored<A> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
        }
    }
}

impl<'src, I, E, A> UnitParser<'src, I, E> for Ignored<A>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
{
    type Output = ();

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, ()> {
        self.parser.go::<Check>(inp)?;
        Ok(M::bind(|| ()))
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::unwrapped`].
pub struct Unwrapped<A, O> {
    pub(crate) parser: A,
    pub(crate) location: Location<'static>,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, O> Copy for Unwrapped<A, O> {}
impl<A: Clone, O> Clone for Unwrapped<A, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            location: self.location,
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, O, U> UnitParser<'src, I, E> for Unwrapped<A, Result<O, U>>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = Result<O, U>>,
    U: fmt::Debug,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let out = self.parser.go::<M>(inp)?;
        Ok(M::map(out, |out| match out {
            Ok(out) => out,
            Err(err) => panic!(
                "called `Result::unwrap` on a `Err(_)` value at {}: {:?}",
                self.location, err
            ),
        }))
    }

    go_extra!(Self::Output);
}

impl<'src, I, E, A, O> UnitParser<'src, I, E> for Unwrapped<A, Option<O>>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = Option<O>>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let out = self.parser.go::<M>(inp)?;
        Ok(M::map(out, |out| match out {
            Some(out) => out,
            None => panic!(
                "called `Option::unwrap` on a `None` value at {}",
                self.location
            ),
        }))
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::memoized`].
#[cfg(feature = "memoization")]
pub struct Memoized<A, O> {
    pub(crate) parser: A,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

#[cfg(feature = "memoization")]
impl<A: Copy, O> Copy for Memoized<A, O> {}
#[cfg(feature = "memoization")]
impl<A: Clone, O> Clone for Memoized<A, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

#[cfg(feature = "memoization")]
impl<'src, I, E, A, O> UnitParser<'src, I, E> for Memoized<A, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    E::Error: Clone,
    A: UnitParser<'src, I, E, Output = O>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let before = inp.cursor();
        // TODO: Don't use address, since this might not be constant?
        let key = (
            I::cursor_location(&before.inner),
            &self.parser as *const _ as *const () as usize,
        );

        match inp.memos.entry(key) {
            hashbrown::hash_map::Entry::Occupied(o) => {
                if let Some(err) = o.get() {
                    let err = err.clone();
                    inp.add_alt_err(&before.inner /*&err.pos*/, err.err);
                } else {
                    let err_span = inp.span_since(&before);
                    // TODO: Is this an appropriate way to handle infinite recursion?
                    inp.add_alt([], None, err_span);
                }
                return Err(());
            }
            hashbrown::hash_map::Entry::Vacant(v) => {
                v.insert(None);
            }
        }

        let res = self.parser.go::<M>(inp);

        if res.is_err() {
            let alt = inp.take_alt();
            inp.memos.insert(key, alt);
        } else {
            inp.memos.remove(&key);
        }

        res
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::then`].
pub struct Then<A, OA, B, OB, E> {
    pub(crate) parser_a: A,
    pub(crate) parser_b: B,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(OA, OB, E)>,
}

impl<A: Copy, OA, B: Copy, OB, E> Copy for Then<A, OA, B, OB, E> {}
impl<A: Clone, OA, B: Clone, OB, E> Clone for Then<A, OA, B, OB, E> {
    fn clone(&self) -> Self {
        Self {
            parser_a: self.parser_a.clone(),
            parser_b: self.parser_b.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, OA, B, OB> UnitParser<'src, I, E> for Then<A, OA, B, OB, E>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = OA>,
    B: UnitParser<'src, I, E, Output = OB>,
{
    type Output = (OA, OB);

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let a = self.parser_a.go::<M>(inp)?;
        let b = self.parser_b.go::<M>(inp)?;
        Ok(M::combine(a, b, |a: A::Output, b: B::Output| (a, b)))
    }

    go_extra!(Self::Output);
}

impl<'src, I, E, A, OA, B, OB> IterParser<'src, I, E> for Then<A, OA, B, OB, E>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: IterParser<'src, I, E>,
    B: IterParser<'src, I, E, Item = A::Item>,
{
    type Item = A::Item;
    type IterState<M: Mode>
        = (A::IterState<M>, Option<B::IterState<M>>)
    where
        I: 'src;

    const NONCONSUMPTION_IS_OK: bool = A::NONCONSUMPTION_IS_OK && B::NONCONSUMPTION_IS_OK;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        Ok((self.parser_a.make_iter::<M>(inp)?, None))
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        state: &mut Self::IterState<M>,
    ) -> IPResult<M, Self::Item> {
        match state {
            (_, Some(b)) => self.parser_b.next(inp, b),
            (a, b) => match self.parser_a.next(inp, a)? {
                Some(a_out) => Ok(Some(a_out)),
                None => {
                    let b = b.insert(self.parser_b.make_iter(inp)?);
                    self.parser_b.next(inp, b)
                }
            },
        }
    }
}

/// See [`UnitParser::ignore_then`].
pub struct IgnoreThen<A, B, E, O> {
    pub(crate) parser_a: A,
    pub(crate) parser_b: B,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(E, O)>,
}

impl<A: Copy, B: Copy, E, O> Copy for IgnoreThen<A, B, E, O> {}
impl<A: Clone, B: Clone, E, O> Clone for IgnoreThen<A, B, E, O> {
    fn clone(&self) -> Self {
        Self {
            parser_a: self.parser_a.clone(),
            parser_b: self.parser_b.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, O> UnitParser<'src, I, E> for IgnoreThen<A, B, E, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    B: UnitParser<'src, I, E, Output = O>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        self.parser_a.go::<Check>(inp)?;
        let b = self.parser_b.go::<M>(inp)?;
        Ok(M::map(b, |b: O| b))
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::then_ignore`].
pub struct ThenIgnore<A, B, E, O> {
    pub(crate) parser_a: A,
    pub(crate) parser_b: B,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(E, O)>,
}

impl<A: Copy, B: Copy, E, O> Copy for ThenIgnore<A, B, E, O> {}
impl<A: Clone, B: Clone, E, O> Clone for ThenIgnore<A, B, E, O> {
    fn clone(&self) -> Self {
        Self {
            parser_a: self.parser_a.clone(),
            parser_b: self.parser_b.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, O> UnitParser<'src, I, E> for ThenIgnore<A, B, E, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
    B: UnitParser<'src, I, E>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let a = self.parser_a.go::<M>(inp)?;
        self.parser_b.go::<Check>(inp)?;
        Ok(M::map(a, |a: O| a))
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::nested_in`].
pub struct NestedIn<A, B, I1, E1, E, O> {
    pub(crate) parser_a: A,
    pub(crate) parser_b: B,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(I1, E1, E, O)>,
}

impl<A: Copy, B: Copy, I1, E1, E, O> Copy for NestedIn<A, B, I1, E1, E, O> {}
impl<A: Clone, B: Clone, I1, E1, E, O> Clone for NestedIn<A, B, I1, E1, E, O> {
    fn clone(&self) -> Self {
        Self {
            parser_a: self.parser_a.clone(),
            parser_b: self.parser_b.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, I1, E1, O> UnitParser<'src, I, E> for NestedIn<A, B, I1, E1, E, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I1, E1, Output = O>,
    B: UnitParser<'src, I, E, Output = I1>,
    I1: Input<'src>,
    E1: ParserExtra<'src, I1, State = E::State, Context = E::Context, Error = E::Error>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let inp2 = self.parser_b.go::<Emit>(inp)?;

        let alt = inp.errors.alt.take();

        #[cfg(feature = "memoization")]
        let mut memos = HashMap::default();
        let (start, mut cache) = inp2.begin();
        let res = inp.with_input(
            start,
            &mut cache,
            &mut Default::default(),
            |inp| (&self.parser_a).then_ignore(end()).go::<M>(inp),
            #[cfg(feature = "memoization")]
            &mut memos,
        );

        // TODO: Translate secondary error offsets too
        let new_alt = inp.errors.alt.take();
        inp.errors.alt = alt;
        if let Some(new_alt) = new_alt {
            inp.add_alt_err(&inp.cursor().inner, new_alt.err);
        }

        res
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::ignore_with_ctx`].
pub struct IgnoreWithCtx<A, B, I, E, O> {
    pub(crate) parser: A,
    pub(crate) then: B,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(I, E, O)>,
}

impl<A: Copy, B: Copy, I, E, O> Copy for IgnoreWithCtx<A, B, I, E, O> {}
impl<A: Clone, B: Clone, I, E, O> Clone for IgnoreWithCtx<A, B, I, E, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            then: self.then.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, O> UnitParser<'src, I, E>
    for IgnoreWithCtx<A, B, I, extra::Full<E::Error, E::State, A::Output>, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    A::Output: 'src,
    B: UnitParser<'src, I, extra::Full<E::Error, E::State, A::Output>, Output = O>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let p1 = self.parser.go::<Emit>(inp)?;
        inp.with_ctx(&p1, |inp| self.then.go::<M>(inp))
    }

    go_extra!(Self::Output);
}

impl<'src, I, E, A, B, O> IterParser<'src, I, E>
    for IgnoreWithCtx<A, B, I, extra::Full<E::Error, E::State, A::Output>, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    A::Output: 'src,
    B: IterParser<'src, I, extra::Full<E::Error, E::State, A::Output>, Item = O>,
{
    type Item = O;

    type IterState<M: Mode>
        = (A::Output, B::IterState<M>)
    where
        I: 'src;

    const NONCONSUMPTION_IS_OK: bool = B::NONCONSUMPTION_IS_OK;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        let out = self.parser.go::<Emit>(inp)?;
        let then = inp.with_ctx(&out, |inp| self.then.make_iter::<M>(inp))?;
        Ok((out, then))
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        state: &mut Self::IterState<M>,
    ) -> IPResult<M, Self::Item> {
        let (ctx, inner_state) = state;

        inp.with_ctx(ctx, |inp| self.then.next(inp, inner_state))
    }
}

/// See [`UnitParser::then_with_ctx`].
pub struct ThenWithCtx<A, OA, B, OB, I, E> {
    pub(crate) parser: A,
    pub(crate) then: B,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(OA, OB, I, E)>,
}

impl<A: Copy, OA, B: Copy, OB, I, E> Copy for ThenWithCtx<A, OA, B, OB, I, E> {}
impl<A: Clone, OA, B: Clone, OB, I, E> Clone for ThenWithCtx<A, OA, B, OB, I, E> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            then: self.then.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, OA, B, OB> UnitParser<'src, I, E>
    for ThenWithCtx<A, OA, B, OB, I, extra::Full<E::Error, E::State, OA>>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = OA>,
    OA: 'src,
    B: UnitParser<'src, I, extra::Full<E::Error, E::State, OA>, Output = OB>,
{
    type Output = (OA, OB);

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let p1 = self.parser.go::<Emit>(inp)?;
        let p2 = inp.with_ctx(&p1, |inp| self.then.go::<M>(inp))?;
        Ok(M::map(p2, |p2| (p1, p2)))
    }

    go_extra!(Self::Output);
}

impl<'src, I, E, A, B, O> IterParser<'src, I, E>
    for ThenWithCtx<A, A::Output, B, O, I, extra::Full<E::Error, E::State, A::Output>>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    A::Output: 'src,
    B: IterParser<'src, I, extra::Full<E::Error, E::State, A::Output>, Item = O>,
{
    type Item = O;
    type IterState<M: Mode>
        = (A::Output, B::IterState<M>)
    where
        I: 'src;

    const NONCONSUMPTION_IS_OK: bool = B::NONCONSUMPTION_IS_OK;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        let out = self.parser.go::<Emit>(inp)?;
        let then = inp.with_ctx(&out, |inp| self.then.make_iter::<M>(inp))?;
        Ok((out, then))
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        state: &mut Self::IterState<M>,
    ) -> IPResult<M, Self::Item> {
        let (ctx, inner_state) = state;

        inp.with_ctx(ctx, |inp| self.then.next(inp, inner_state))
    }
}

/// See [`UnitParser::with_ctx`].
pub struct WithCtx<A, Ctx, O> {
    pub(crate) parser: A,
    pub(crate) ctx: Ctx,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, Ctx: Copy, O> Copy for WithCtx<A, Ctx, O> {}
impl<A: Clone, Ctx: Clone, O> Clone for WithCtx<A, Ctx, O> {
    fn clone(&self) -> Self {
        WithCtx {
            parser: self.parser.clone(),
            ctx: self.ctx.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, Ctx, O> UnitParser<'src, I, E> for WithCtx<A, Ctx, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, extra::Full<E::Error, E::State, Ctx>, Output = O>,
    Ctx: 'src,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        inp.with_ctx(&self.ctx, |inp| self.parser.go::<M>(inp))
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::with_state`].
pub struct WithState<A, State, O> {
    pub(crate) parser: A,
    pub(crate) state: State,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, Ctx: Copy, O> Copy for WithState<A, Ctx, O> {}
impl<A: Clone, Ctx: Clone, O> Clone for WithState<A, Ctx, O> {
    fn clone(&self) -> Self {
        WithState {
            parser: self.parser.clone(),
            state: self.state.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, State, O> UnitParser<'src, I, E> for WithState<A, State, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, extra::Full<E::Error, State, E::Context>, Output = O>,
    State: 'src + Clone + Inspector<'src, I>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        inp.with_state(&mut self.state.clone(), |inp| self.parser.go::<M>(inp))
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::delimited_by`].
pub struct DelimitedBy<A, B, C, O> {
    pub(crate) parser: A,
    pub(crate) start: B,
    pub(crate) end: C,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, B: Copy, C: Copy, O> Copy for DelimitedBy<A, B, C, O> {}
impl<A: Clone, B: Clone, C: Clone, O> Clone for DelimitedBy<A, B, C, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            start: self.start.clone(),
            end: self.end.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, C, O> UnitParser<'src, I, E> for DelimitedBy<A, B, C, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
    B: UnitParser<'src, I, E>,
    C: UnitParser<'src, I, E>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        self.start.go::<Check>(inp)?;
        let a = self.parser.go::<M>(inp)?;
        self.end.go::<Check>(inp)?;
        Ok(a)
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::padded_by`].
pub struct PaddedBy<A, B, O> {
    pub(crate) parser: A,
    pub(crate) padding: B,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, B: Copy, O> Copy for PaddedBy<A, B, O> {}
impl<A: Clone, B: Clone, O> Clone for PaddedBy<A, B, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            padding: self.padding.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, O> UnitParser<'src, I, E> for PaddedBy<A, B, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
    B: UnitParser<'src, I, E>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        self.padding.go::<Check>(inp)?;
        let a = self.parser.go::<M>(inp)?;
        self.padding.go::<Check>(inp)?;
        Ok(a)
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::or`].
pub struct Or<A, B, O> {
    pub(crate) choice: crate::primitive::Choice<(A, B)>,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, B: Copy, O> Copy for Or<A, B, O> {}
impl<A: Clone, B: Clone, O> Clone for Or<A, B, O> {
    fn clone(&self) -> Self {
        Self {
            choice: self.choice.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, O> UnitParser<'src, I, E> for Or<A, B, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
    B: UnitParser<'src, I, E, Output = O>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        self.choice.go::<M>(inp)
    }

    go_extra!(Self::Output);
}

/// Configuration for [`UnitParser::repeated`], used in [`ConfigParser::configure`].
#[derive(Default)]
pub struct RepeatedCfg {
    at_least: Option<usize>,
    at_most: Option<usize>,
}

impl RepeatedCfg {
    /// Set the minimum number of repetitions accepted
    pub fn at_least(mut self, n: usize) -> Self {
        self.at_least = Some(n);
        self
    }

    /// Set the maximum number of repetitions accepted
    pub fn at_most(mut self, n: usize) -> Self {
        self.at_most = Some(n);
        self
    }

    /// Set an exact number of repetitions to accept
    pub fn exactly(mut self, n: usize) -> Self {
        self.at_least = Some(n);
        self.at_most = Some(n);
        self
    }
}

/// See [`UnitParser::repeated`].
pub struct Repeated<A, I, E, O> {
    pub(crate) parser: A,
    pub(crate) at_least: usize,
    // Slightly evil: Should be `Option<usize>`, but we encode `!0` as 'no cap' because it's so large
    pub(crate) at_most: u64,
    #[cfg(debug_assertions)]
    pub(crate) location: Location<'static>,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(I, E, O)>,
}

impl<A: Copy, I, E, O> Copy for Repeated<A, I, E, O> {}
impl<A: Clone, I, E, O> Clone for Repeated<A, I, E, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            at_least: self.at_least,
            at_most: self.at_most,
            #[cfg(debug_assertions)]
            location: self.location,
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, A, I, E, O> Repeated<A, I, E, O>
where
    A: UnitParser<'src, I, E, Output = O>,
    I: Input<'src>,
    E: ParserExtra<'src, I>,
{
    /// Require that the pattern appear at least a minimum number of times.
    pub fn at_least(self, at_least: usize) -> Self {
        Self { at_least, ..self }
    }

    /// Require that the pattern appear at most a maximum number of times.
    pub fn at_most(self, at_most: usize) -> Self {
        Self {
            at_most: at_most as u64,
            ..self
        }
    }

    /// Require that the pattern appear exactly the given number of times.
    ///
    /// ```
    /// # use chumsky::prelude::*;
    /// let ring = just::<_, _, extra::Err<Simple<char>>>('O');
    ///
    /// let for_the_elves = ring
    ///     .repeated()
    ///     .exactly(3)
    ///     .collect::<Vec<_>>();
    ///
    /// let for_the_dwarves = ring
    ///     .repeated()
    ///     .exactly(7)
    ///     .collect::<Vec<_>>();
    ///
    /// let for_the_humans = ring
    ///     .repeated()
    ///     .exactly(9)
    ///     .collect::<Vec<_>>();
    ///
    /// let for_sauron = ring
    ///     .repeated()
    ///     .exactly(1)
    ///     .collect::<Vec<_>>();
    ///
    /// let rings = for_the_elves
    ///     .then(for_the_dwarves)
    ///     .then(for_the_humans)
    ///     .then(for_sauron);
    ///
    /// assert!(rings.parse("OOOOOOOOOOOOOOOOOOO").has_errors()); // Too few rings!
    /// assert!(rings.parse("OOOOOOOOOOOOOOOOOOOOO").has_errors()); // Too many rings!
    /// // The perfect number of rings
    /// assert_eq!(
    ///     rings.parse("OOOOOOOOOOOOOOOOOOOO").into_result(),
    ///     Ok(((((vec!['O'; 3]), vec!['O'; 7]), vec!['O'; 9]), vec!['O'; 1])),
    /// );
    /// ````
    pub fn exactly(self, exactly: usize) -> Self {
        Self {
            at_least: exactly,
            at_most: exactly as u64,
            ..self
        }
    }
}

impl<'src, I, E, A, O> UnitParser<'src, I, E> for Repeated<A, I, E, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
{
    type Output = ();

    #[inline(always)]
    #[allow(clippy::nonminimal_bool)] // TODO: Remove this, lint is currently buggy
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, ()> {
        if self.at_most == !0 && self.at_least == 0 {
            loop {
                let before = inp.save();
                match self.parser.go::<Check>(inp) {
                    Ok(()) => {}
                    Err(()) => {
                        inp.rewind(before);
                        break Ok(M::bind(|| ()));
                    }
                }
                #[cfg(debug_assertions)]
                debug_assert!(
                    *before.cursor() != inp.cursor(),
                    "found Repeated combinator making no progress at {}",
                    self.location,
                );
            }
        } else {
            let mut state = self.make_iter::<Check>(inp)?;
            loop {
                #[cfg(debug_assertions)]
                let before = inp.cursor();
                match self.next::<Check>(inp, &mut state) {
                    Ok(Some(())) => {}
                    Ok(None) => break Ok(M::bind(|| ())),
                    // TODO: Technically we should be rewinding here: as-is, this is invalid since errorring parsers
                    // are permitted to leave input state unspecified. Really, unwinding should occur *here* and not in
                    // `next`.
                    Err(()) => break Err(()),
                }
                #[cfg(debug_assertions)]
                debug_assert!(
                    before != inp.cursor(),
                    "found Repeated combinator making no progress at {}",
                    self.location,
                );
            }
        }
    }

    go_extra!(Self::Output);
}

impl<'src, A, I, E, O> IterParser<'src, I, E> for Repeated<A, I, E, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
{
    type Item = O;
    type IterState<M: Mode> = usize;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        _inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        Ok(0)
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        count: &mut Self::IterState<M>,
    ) -> IPResult<M, Self::Item> {
        if *count as u64 >= self.at_most {
            return Ok(None);
        }

        let before = inp.save();
        match self.parser.go::<M>(inp) {
            Ok(item) => {
                *count += 1;
                Ok(Some(item))
            }
            Err(()) => {
                inp.rewind(before);
                if *count >= self.at_least {
                    Ok(None)
                } else {
                    Err(())
                }
            }
        }
    }
}

impl<'src, A, I, E, O> ConfigIterParser<'src, I, E> for Repeated<A, I, E, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
{
    type Config = RepeatedCfg;

    #[inline(always)]
    fn next_cfg<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        count: &mut Self::IterState<M>,
        cfg: &Self::Config,
    ) -> IPResult<M, Self::Item> {
        let at_most = cfg.at_most.map(|x| x as u64).unwrap_or(self.at_most);
        let at_least = cfg.at_least.unwrap_or(self.at_least);

        if *count as u64 >= at_most {
            return Ok(None);
        }

        let before = inp.save();
        match self.parser.go::<M>(inp) {
            Ok(item) => {
                *count += 1;
                Ok(Some(item))
            }
            Err(()) => {
                inp.rewind(before);
                if *count >= at_least {
                    Ok(None)
                } else {
                    Err(())
                }
            }
        }
    }
}

/// See [`UnitParser::separated_by`].
pub struct SeparatedBy<A, B, I, E, O> {
    pub(crate) parser: A,
    pub(crate) separator: B,
    pub(crate) at_least: usize,
    // Slightly evil: Should be `Option<usize>`, but we encode `!0` as 'no cap' because it's so large
    pub(crate) at_most: u64,
    pub(crate) allow_leading: bool,
    pub(crate) allow_trailing: bool,
    #[cfg(debug_assertions)]
    pub(crate) location: Location<'static>,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(I, E, O)>,
}

impl<A: Copy, B: Copy, I, E, O> Copy for SeparatedBy<A, B, I, E, O> {}
impl<A: Clone, B: Clone, I, E, O> Clone for SeparatedBy<A, B, I, E, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            separator: self.separator.clone(),
            at_least: self.at_least,
            at_most: self.at_most,
            allow_leading: self.allow_leading,
            allow_trailing: self.allow_trailing,
            #[cfg(debug_assertions)]
            location: self.location,
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, O> SeparatedBy<A, B, I, E, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
    B: UnitParser<'src, I, E>,
{
    /// Require that the pattern appear at least a minimum number of times.
    ///
    /// ```
    /// # use chumsky::prelude::*;
    /// let numbers = just::<_, _, extra::Err<Simple<char>>>('-')
    ///     .separated_by(just('.'))
    ///     .at_least(2)
    ///     .collect::<Vec<_>>();
    ///
    /// assert!(numbers.parse("").has_errors());
    /// assert!(numbers.parse("-").has_errors());
    /// assert_eq!(numbers.parse("-.-").into_result(), Ok(vec!['-', '-']));
    /// ````
    pub fn at_least(self, at_least: usize) -> Self {
        Self { at_least, ..self }
    }

    /// Require that the pattern appear at most a maximum number of times.
    ///
    /// ```
    /// # use chumsky::prelude::*;
    /// let row_4 = text::int::<_, extra::Err<Simple<char>>>(10)
    ///     .padded()
    ///     .separated_by(just(','))
    ///     .at_most(4)
    ///     .collect::<Vec<_>>();
    ///
    /// let matrix_4x4 = row_4
    ///     .separated_by(just(','))
    ///     .at_most(4)
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(
    ///     matrix_4x4.parse("0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15").into_result(),
    ///     Ok(vec![
    ///         vec!["0", "1", "2", "3"],
    ///         vec!["4", "5", "6", "7"],
    ///         vec!["8", "9", "10", "11"],
    ///         vec!["12", "13", "14", "15"],
    ///     ]),
    /// );
    /// ````
    pub fn at_most(self, at_most: usize) -> Self {
        Self {
            at_most: at_most as u64,
            ..self
        }
    }

    /// Require that the pattern appear exactly the given number of times.
    ///
    /// ```
    /// # use chumsky::prelude::*;
    /// let coordinate_3d = text::int::<_, extra::Err<Simple<char>>>(10)
    ///     .padded()
    ///     .separated_by(just(','))
    ///     .exactly(3)
    ///     .collect::<Vec<_>>();
    ///
    /// // Not enough elements
    /// assert!(coordinate_3d.parse("4, 3").has_errors());
    /// // Too many elements
    /// assert!(coordinate_3d.parse("7, 2, 13, 4").has_errors());
    /// // Just the right number of elements
    /// assert_eq!(coordinate_3d.parse("5, 0, 12").into_result(), Ok(vec!["5", "0", "12"]));
    /// ````
    pub fn exactly(self, exactly: usize) -> Self {
        Self {
            at_least: exactly,
            at_most: exactly as u64,
            ..self
        }
    }

    /// Allow a leading separator to appear before the first item.
    ///
    /// Note that even if no items are parsed, a leading separator *is* permitted.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chumsky::prelude::*;
    /// let r#enum = text::ascii::keyword::<_, _, extra::Err<Simple<char>>>("enum")
    ///     .padded()
    ///     .ignore_then(text::ascii::ident()
    ///         .padded()
    ///         .separated_by(just('|'))
    ///         .allow_leading()
    ///         .collect::<Vec<_>>());
    ///
    /// assert_eq!(r#enum.parse("enum True | False").into_result(), Ok(vec!["True", "False"]));
    /// assert_eq!(r#enum.parse("
    ///     enum
    ///     | True
    ///     | False
    /// ").into_result(), Ok(vec!["True", "False"]));
    /// ```
    pub fn allow_leading(self) -> Self {
        Self {
            allow_leading: true,
            ..self
        }
    }

    /// Allow a trailing separator to appear after the last item.
    ///
    /// Note that if no items are parsed, no leading separator is permitted.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chumsky::prelude::*;
    /// let numbers = text::int::<_, extra::Err<Simple<char>>>(10)
    ///     .padded()
    ///     .separated_by(just(','))
    ///     .allow_trailing()
    ///     .collect::<Vec<_>>()
    ///     .delimited_by(just('('), just(')'));
    ///
    /// assert_eq!(numbers.parse("(1, 2)").into_result(), Ok(vec!["1", "2"]));
    /// assert_eq!(numbers.parse("(1, 2,)").into_result(), Ok(vec!["1", "2"]));
    /// ```
    pub fn allow_trailing(self) -> Self {
        Self {
            allow_trailing: true,
            ..self
        }
    }
}

impl<'src, I, E, A, B, O> IterParser<'src, I, E> for SeparatedBy<A, B, I, E, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
    B: UnitParser<'src, I, E>,
{
    type Item = O;
    type IterState<M: Mode>
        = usize
    where
        I: 'src;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        _inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        Ok(0)
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        state: &mut Self::IterState<M>,
    ) -> IPResult<M, Self::Item> {
        if *state as u64 >= self.at_most {
            return Ok(None);
        }

        let before_separator = inp.save();
        if *state == 0 && self.allow_leading {
            if self.separator.go::<Check>(inp).is_err() {
                inp.rewind(before_separator.clone());
            }
        } else if *state > 0 {
            match self.separator.go::<Check>(inp) {
                Ok(()) => {
                    // Do nothing
                }
                Err(()) if *state < self.at_least => {
                    inp.rewind(before_separator);
                    return Err(());
                }
                Err(()) => {
                    inp.rewind(before_separator);
                    return Ok(None);
                }
            }
        }

        let before_item = inp.save();
        match self.parser.go::<M>(inp) {
            Ok(item) => {
                *state += 1;
                Ok(Some(item))
            }
            Err(()) if *state < self.at_least => {
                // We have errored before we have reached the count,
                // and therefore should return this error, as we are
                // still expecting items
                inp.rewind(before_separator);
                Err(())
            }
            Err(()) => {
                // We are not expecting any more items, so it is okay
                // for it to fail.

                // though if we don't allow trailing, we shouldn't have
                // consumed the separator, so we need to rewind it.
                if self.allow_trailing {
                    inp.rewind(before_item);
                } else {
                    inp.rewind(before_separator);
                }
                Ok(None)
            }
        }
    }
}

impl<'src, I, E, A, B, O> UnitParser<'src, I, E> for SeparatedBy<A, B, I, E, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
    B: UnitParser<'src, I, E>,
{
    type Output = ();

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, ()> {
        let mut state = self.make_iter::<Check>(inp)?;
        loop {
            #[cfg(debug_assertions)]
            let before = inp.cursor();
            match self.next::<Check>(inp, &mut state) {
                Ok(Some(())) => {}
                Ok(None) => break Ok(M::bind(|| ())),
                // TODO: Technically we should be rewinding here: as-is, this is invalid since errorring parsers
                // are permitted to leave input state unspecified. Really, unwinding should occur *here* and not in
                // `next`.
                Err(()) => break Err(()),
            }
            #[cfg(debug_assertions)]
            debug_assert!(
                before != inp.cursor(),
                "found SeparatedBy combinator making no progress at {}",
                self.location,
            );
        }
    }

    go_extra!(Self::Output);
}

/// See [`IterParser::enumerate`].
pub struct Enumerate<A, OA> {
    pub(crate) parser: A,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<OA>,
}

impl<A: Copy, OA> Copy for Enumerate<A, OA> {}
impl<A: Clone, OA> Clone for Enumerate<A, OA> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, OA> IterParser<'src, I, E> for Enumerate<A, OA>
where
    A: IterParser<'src, I, E, Item = OA>,
    I: Input<'src>,
    E: ParserExtra<'src, I>,
{
    type Item = (usize, OA);
    type IterState<M: Mode>
        = (usize, A::IterState<M>)
    where
        I: 'src;

    const NONCONSUMPTION_IS_OK: bool = A::NONCONSUMPTION_IS_OK;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        Ok((0, A::make_iter(&self.parser, inp)?))
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        state: &mut Self::IterState<M>,
    ) -> IPResult<M, Self::Item> {
        let out = self
            .parser
            .next(inp, &mut state.1)?
            .map(|out| M::map(out, |out| (state.0, out)));
        state.0 += 1;
        Ok(out)
    }
}

/// See [`IterParser::collect`].
pub struct Collect<A, O> {
    pub(crate) parser: A,
    #[cfg(debug_assertions)]
    pub(crate) location: Location<'static>,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, O> Copy for Collect<A, O> {}
impl<A: Clone, O> Clone for Collect<A, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            #[cfg(debug_assertions)]
            location: self.location,
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, O> UnitParser<'src, I, E> for Collect<A, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: IterParser<'src, I, E>,
    O: Container<A::Item>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let mut output = M::bind::<O, _>(|| O::default());
        let mut iter_state = self.parser.make_iter::<M>(inp)?;
        #[cfg(debug_assertions)]
        let mut i = 0;
        loop {
            #[cfg(debug_assertions)]
            let before = inp.cursor();
            match self.parser.next::<M>(inp, &mut iter_state) {
                Ok(Some(out)) => {
                    M::combine_mut(&mut output, out, |output: &mut O, item| output.push(item));
                }
                Ok(None) => break Ok(output),
                Err(()) => break Err(()),
            }
            // We only check after the second iteration because that's when we *must* have consumed both item
            // and separator.
            #[cfg(debug_assertions)]
            if !A::NONCONSUMPTION_IS_OK {
                if i >= 1 {
                    debug_assert!(
                        before != inp.cursor(),
                        "found Collect combinator making no progress at {}",
                        self.location,
                    );
                }
                i += 1;
            }
        }
    }

    go_extra!(Self::Output);
}

/// See [`IterParser::collect_exactly`]
pub struct CollectExactly<A, O> {
    pub(crate) parser: A,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, O> Copy for CollectExactly<A, O> {}
impl<A: Clone, O> Clone for CollectExactly<A, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, O> UnitParser<'src, I, E> for CollectExactly<A, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: IterParser<'src, I, E>,
    O: ContainerExactly<A::Item>,
{
    type Output = O;

    #[inline]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        // let before = inp.cursor();
        let mut output = M::bind(|| O::uninit());
        let mut iter_state = self.parser.make_iter::<M>(inp)?;
        for idx in 0..O::LEN {
            match self.parser.next::<M>(inp, &mut iter_state) {
                Ok(Some(out)) => {
                    M::combine_mut(&mut output, out, |c, out| O::write(c, idx, out));
                }
                Ok(None) => {
                    // let span = inp.span_since(&before);
                    // We don't add an alt here because we assume the inner parser will. Is this safe to assume?
                    // inp.add_alt([ExpectedMoreElements(Some(C::LEN - idx))], None, span);
                    // SAFETY: We're guaranteed to have initialized up to `idx` values
                    M::map(output, |mut output| unsafe {
                        O::drop_before(&mut output, idx)
                    });
                    return Err(());
                }
                Err(()) => {
                    // SAFETY: We're guaranteed to have initialized up to `idx` values
                    M::map(output, |mut output| unsafe {
                        O::drop_before(&mut output, idx)
                    });
                    return Err(());
                }
            }
        }
        // SAFETY: If we reach this point, we guarantee to have initialized C::LEN values
        Ok(M::map(output, |output| unsafe { O::take(output) }))
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::or_not`].
pub struct OrNot<A, OA> {
    pub(crate) parser: A,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<OA>,
}

impl<A: Copy, OA> Copy for OrNot<A, OA> {}
impl<A: Clone, OA> Clone for OrNot<A, OA> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, OA> UnitParser<'src, I, E> for OrNot<A, OA>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = OA>,
{
    type Output = Option<OA>;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let before = inp.save();
        Ok(match self.parser.go::<M>(inp) {
            Ok(out) => M::map::<A::Output, _, _>(out, Some),
            Err(()) => {
                inp.rewind(before);
                M::bind::<Self::Output, _>(|| None)
            }
        })
    }

    go_extra!(Self::Output);
}

impl<'src, I, E, A, O> IterParser<'src, I, E> for OrNot<A, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
{
    type Item = O;
    type IterState<M: Mode> = bool;

    const NONCONSUMPTION_IS_OK: bool = true;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        _inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        Ok(false)
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        finished: &mut Self::IterState<M>,
    ) -> IPResult<M, Self::Item> {
        if *finished {
            return Ok(None);
        }

        let before = inp.save();
        match self.parser.go::<M>(inp) {
            Ok(item) => {
                *finished = true;
                Ok(Some(item))
            }
            Err(()) => {
                inp.rewind(before);
                *finished = true;
                Ok(None)
            }
        }
    }
}

/// See [`UnitParser::not`].
pub struct Not<A> {
    pub(crate) parser: A,
}

impl<A: Copy> Copy for Not<A> {}
impl<A: Clone> Clone for Not<A> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
        }
    }
}

impl<'src, I, E, A> UnitParser<'src, I, E> for Not<A>
where
    I: ValueInput<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
{
    type Output = ();

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, ()> {
        let before = inp.save();

        let alt = inp.errors.alt.take();

        let result = self.parser.go::<Check>(inp);
        let result_span = inp.span_since(before.cursor());
        inp.rewind(before);

        inp.errors.alt = alt;

        match result {
            Ok(()) => {
                let found = inp.next_inner();
                inp.add_alt(
                    [DefaultExpected::SomethingElse],
                    found.map(|f| f.into()),
                    result_span,
                );
                Err(())
            }
            Err(()) => Ok(M::bind(|| ())),
        }
    }

    go_extra!(Self::Output);
}

/// See [`IterParser::flatten`].
#[cfg(feature = "nightly")]
pub struct Flatten<A, O> {
    pub(crate) parser: A,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

#[cfg(feature = "nightly")]
impl<A: Copy, O> Copy for Flatten<A, O> {}
#[cfg(feature = "nightly")]
impl<A: Clone, O> Clone for Flatten<A, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

#[cfg(feature = "nightly")]
impl<'src, I, E, A, O> IterParser<'src, I, E> for Flatten<A, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: IterParser<'src, I, E>,
    A::Item: IntoIterator<Item = O>,
{
    type Item = O;
    type IterState<M: Mode> = (
        A::IterState<M>,
        Option<M::Output<<<A as IterParser<'src, I, E>>::Item as IntoIterator>::IntoIter>>,
    );

    // A::NONCONSUMPTION_IS_OK cannot be used because if we are iterating
    // over O, we are not consuming any input (input has probably
    // already been comsumed when constructing O)
    const NONCONSUMPTION_IS_OK: bool = true;

    #[inline(always)]
    fn make_iter<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
    ) -> PResult<Emit, Self::IterState<M>> {
        Ok((self.parser.make_iter(inp)?, None))
    }

    #[inline(always)]
    fn next<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        (st, iter): &mut Self::IterState<M>,
    ) -> IPResult<M, Self::Item> {
        if let Some(item) = iter
            .as_mut()
            .and_then(|i| M::get_or(M::map(M::from_mut(i), |i| i.next()), || None))
        {
            return Ok(Some(M::bind(move || item)));
        }

        // TODO: Debug looping check
        loop {
            let before = inp.save();
            match self.parser.next::<M>(inp, st) {
                Ok(Some(item)) => match M::get_or(
                    M::map(
                        M::from_mut(iter.insert(M::map(item, |i| i.into_iter()))),
                        |i| i.next().map(Some),
                    ),
                    || Some(None),
                ) {
                    Some(Some(item)) => break Ok(Some(M::bind(move || item))),
                    Some(None) => break Ok(Some(M::bind(|| unreachable!()))),
                    None => continue,
                },
                Ok(None) => break Ok(None),
                Err(()) => {
                    inp.rewind(before);
                    break Err(());
                }
            }
        }
    }
}

/// See [`UnitParser::and_is`].
pub struct AndIs<A, B, O> {
    pub(crate) parser_a: A,
    pub(crate) parser_b: B,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, B: Copy, O> Copy for AndIs<A, B, O> {}
impl<A: Clone, B: Clone, O> Clone for AndIs<A, B, O> {
    fn clone(&self) -> Self {
        Self {
            parser_a: self.parser_a.clone(),
            parser_b: self.parser_b.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, O> UnitParser<'src, I, E> for AndIs<A, B, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
    B: UnitParser<'src, I, E>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let before = inp.save().clone();
        match self.parser_a.go::<M>(inp) {
            Ok(out) => {
                // A succeeded -- go back to the beginning and try B
                let after = inp.save();
                inp.rewind(before);

                match self.parser_b.go::<Check>(inp) {
                    Ok(()) => {
                        // B succeeded -- go to the end of A and return its output
                        inp.rewind(after);
                        Ok(out)
                    }
                    Err(()) => {
                        // B failed -- go back to the beginning and fail
                        Err(())
                    }
                }
            }
            Err(()) => {
                // A failed -- go back to the beginning and fail
                inp.rewind(before);
                Err(())
            }
        }
    }

    go_extra!(Self::Output);
}

/// See [`IterParser::foldr`].
pub struct Foldr<F, A, B, E, O> {
    pub(crate) parser_a: A,
    pub(crate) parser_b: B,
    pub(crate) folder: F,
    #[cfg(debug_assertions)]
    pub(crate) location: Location<'static>,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(E, O)>,
}

impl<F: Copy, A: Copy, B: Copy, E, O> Copy for Foldr<F, A, B, E, O> {}
impl<F: Clone, A: Clone, B: Clone, E, O> Clone for Foldr<F, A, B, E, O> {
    fn clone(&self) -> Self {
        Self {
            parser_a: self.parser_a.clone(),
            parser_b: self.parser_b.clone(),
            folder: self.folder.clone(),
            #[cfg(debug_assertions)]
            location: self.location,
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, F, O> UnitParser<'src, I, E> for Foldr<F, A, B, E, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: IterParser<'src, I, E>,
    B: UnitParser<'src, I, E, Output = O>,
    F: Fn(A::Item, O) -> O,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
    where
        Self: Sized,
    {
        let mut a_out = M::bind(|| Vec::new());
        let mut iter_state = self.parser_a.make_iter::<M>(inp)?;
        loop {
            #[cfg(debug_assertions)]
            let before = inp.cursor();
            match self.parser_a.next::<M>(inp, &mut iter_state) {
                Ok(Some(out)) => {
                    M::combine_mut(&mut a_out, out, |a_out, item| a_out.push(item));
                }
                Ok(None) => break,
                Err(()) => return Err(()),
            }
            #[cfg(debug_assertions)]
            if !A::NONCONSUMPTION_IS_OK {
                debug_assert!(
                    before != inp.cursor(),
                    "found Foldr combinator making no progress at {}",
                    self.location,
                );
            }
        }

        let b_out = self.parser_b.go::<M>(inp)?;

        Ok(M::combine(a_out, b_out, |a_out, b_out| {
            a_out.into_iter().rfold(b_out, |b, a| (self.folder)(a, b))
        }))
    }

    go_extra!(Self::Output);
}

/// See [`IterParser::foldr_with`].
pub struct FoldrWith<F, A, B, E, O> {
    pub(crate) parser_a: A,
    pub(crate) parser_b: B,
    pub(crate) folder: F,
    #[cfg(debug_assertions)]
    pub(crate) location: Location<'static>,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(E, O)>,
}

impl<F: Copy, A: Copy, B: Copy, E, O> Copy for FoldrWith<F, A, B, E, O> {}
impl<F: Clone, A: Clone, B: Clone, E, O> Clone for FoldrWith<F, A, B, E, O> {
    fn clone(&self) -> Self {
        Self {
            parser_a: self.parser_a.clone(),
            parser_b: self.parser_b.clone(),
            folder: self.folder.clone(),
            #[cfg(debug_assertions)]
            location: self.location,
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, F, O> UnitParser<'src, I, E> for FoldrWith<F, A, B, E, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: IterParser<'src, I, E>,
    B: UnitParser<'src, I, E, Output = O>,
    F: Fn(A::Item, O, &mut MapExtra<'src, '_, I, E>) -> O,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
    where
        Self: Sized,
    {
        let mut a_out = M::bind(Vec::new);
        let mut iter_state = self.parser_a.make_iter::<M>(inp)?;
        loop {
            let before = inp.cursor();
            match self.parser_a.next::<M>(inp, &mut iter_state) {
                Ok(Some(out)) => {
                    M::combine_mut(&mut a_out, out, |a_out, item| {
                        a_out.push((item, before.clone()))
                    });
                }
                Ok(None) => break,
                Err(()) => return Err(()),
            }
            #[cfg(debug_assertions)]
            if !A::NONCONSUMPTION_IS_OK {
                debug_assert!(
                    before != inp.cursor(),
                    "found FoldrWithState combinator making no progress at {}",
                    self.location,
                );
            }
        }

        let b_out = self.parser_b.go::<M>(inp)?;

        Ok(M::combine(a_out, b_out, |a_out, b_out| {
            a_out.into_iter().rfold(b_out, |b, (a, before)| {
                (self.folder)(a, b, &mut MapExtra::new(&before, inp))
            })
        }))
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::foldl`].
pub struct Foldl<F, A, B, E, O> {
    pub(crate) parser_a: A,
    pub(crate) parser_b: B,
    pub(crate) folder: F,
    #[cfg(debug_assertions)]
    pub(crate) location: Location<'static>,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(E, O)>,
}

impl<F: Copy, A: Copy, B: Copy, E, O> Copy for Foldl<F, A, B, E, O> {}
impl<F: Clone, A: Clone, B: Clone, E, O> Clone for Foldl<F, A, B, E, O> {
    fn clone(&self) -> Self {
        Self {
            parser_a: self.parser_a.clone(),
            parser_b: self.parser_b.clone(),
            folder: self.folder.clone(),
            #[cfg(debug_assertions)]
            location: self.location,
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, F, O> UnitParser<'src, I, E> for Foldl<F, A, B, E, O>
where
    I: Input<'src>,
    A: UnitParser<'src, I, E, Output = O>,
    B: IterParser<'src, I, E>,
    E: ParserExtra<'src, I>,
    F: Fn(O, B::Item) -> O,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
    where
        Self: Sized,
    {
        let mut out = self.parser_a.go::<M>(inp)?;
        let mut iter_state = self.parser_b.make_iter::<M>(inp)?;
        loop {
            #[cfg(debug_assertions)]
            let before = inp.cursor();
            match self.parser_b.next::<M>(inp, &mut iter_state) {
                Ok(Some(b_out)) => {
                    out = M::combine(out, b_out, |out, b_out| (self.folder)(out, b_out));
                }
                Ok(None) => break Ok(out),
                Err(()) => break Err(()),
            }
            #[cfg(debug_assertions)]
            if !B::NONCONSUMPTION_IS_OK {
                debug_assert!(
                    before != inp.cursor(),
                    "found Foldl combinator making no progress at {}",
                    self.location,
                );
            }
        }
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::foldl_with`].
pub struct FoldlWith<F, A, B, E, O> {
    pub(crate) parser_a: A,
    pub(crate) parser_b: B,
    pub(crate) folder: F,
    #[cfg(debug_assertions)]
    pub(crate) location: Location<'static>,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<(E, O)>,
}

impl<F: Copy, A: Copy, B: Copy, E, O> Copy for FoldlWith<F, A, B, E, O> {}
impl<F: Clone, A: Clone, B: Clone, E, O> Clone for FoldlWith<F, A, B, E, O> {
    fn clone(&self) -> Self {
        Self {
            parser_a: self.parser_a.clone(),
            parser_b: self.parser_b.clone(),
            folder: self.folder.clone(),
            #[cfg(debug_assertions)]
            location: self.location,
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, B, F, O> UnitParser<'src, I, E> for FoldlWith<F, A, B, E, O>
where
    I: Input<'src>,
    A: UnitParser<'src, I, E, Output = O>,
    B: IterParser<'src, I, E>,
    E: ParserExtra<'src, I>,
    F: Fn(O, B::Item, &mut MapExtra<'src, '_, I, E>) -> O,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
    where
        Self: Sized,
    {
        let before_all = inp.cursor();
        let mut out = self.parser_a.go::<M>(inp)?;
        let mut iter_state = self.parser_b.make_iter::<M>(inp)?;
        loop {
            #[cfg(debug_assertions)]
            let before = inp.cursor();
            match self.parser_b.next::<M>(inp, &mut iter_state) {
                Ok(Some(b_out)) => {
                    out = M::combine(out, b_out, |out, b_out| {
                        (self.folder)(out, b_out, &mut MapExtra::new(&before_all, inp))
                    })
                }
                Ok(None) => break Ok(out),
                Err(()) => break Err(()),
            }
            #[cfg(debug_assertions)]
            if !B::NONCONSUMPTION_IS_OK {
                debug_assert!(
                    before != inp.cursor(),
                    "found FoldlWithState combinator making no progress at {}",
                    self.location,
                );
            }
        }
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::rewind`].
#[must_use]
pub struct Rewind<A, O> {
    pub(crate) parser: A,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, O> Copy for Rewind<A, O> {}
impl<A: Clone, O> Clone for Rewind<A, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, O> UnitParser<'src, I, E> for Rewind<A, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        let before = inp.save();
        match self.parser.go::<M>(inp) {
            Ok(out) => {
                inp.rewind(before);
                Ok(out)
            }
            Err(()) => Err(()),
        }
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::map_err`].
pub struct MapErr<A, F, O> {
    pub(crate) parser: A,
    pub(crate) mapper: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, F: Copy, O> Copy for MapErr<A, F, O> {}
impl<A: Clone, F: Clone, O> Clone for MapErr<A, F, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            mapper: self.mapper.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, F, O> UnitParser<'src, I, E> for MapErr<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
    F: Fn(E::Error) -> E::Error,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
    where
        Self: Sized,
    {
        (&self.parser)
            .map_err_with_state(|e, _, _| (self.mapper)(e))
            .go::<M>(inp)
    }

    go_extra!(Self::Output);
}

// /// See [`UnitParser::map_err_with_span`].
// #[derive(Copy, Clone)]
// pub struct MapErrWithSpan<A, F> {
//     pub(crate) parser: A,
//     pub(crate) mapper: F,
// }

// impl<'src, I, E, A, F> Parser<'src, I, E> for MapErrWithSpan<A, F>
// where
//     I: Input<'src>,
//     E: ParserExtra<'src, I>,
//     A: Parser<'src, I, E>,
//     F: Fn(E::Error, I::Span) -> E::Error,
// {
//     type Output = A::Output;
//
//     #[inline(always)]
//     fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
//     where
//         Self: Sized,
//     {
//         let start = inp.cursor();
//         let res = self.parser.go::<M>(inp);

//         if res.is_err() {
//             let mut e = inp.take_alt();
//             let span = inp.span_since(start);
//             e.err = (self.mapper)(e.err, span);
//             inp.errors.alt = Some(e);
//         }

//         res
//     }

//     go_extra!(Self::Output);
// }

// TODO: Remove combinator, replace with map_err_with
/// See [`UnitParser::map_err_with_state`].
pub struct MapErrWithState<A, F, O> {
    pub(crate) parser: A,
    pub(crate) mapper: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, F: Copy, O> Copy for MapErrWithState<A, F, O> {}
impl<A: Clone, F: Clone, O> Clone for MapErrWithState<A, F, O> {
    fn clone(&self) -> Self {
        Self {
            parser: self.parser.clone(),
            mapper: self.mapper.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, F, O> UnitParser<'src, I, E> for MapErrWithState<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
    F: Fn(E::Error, I::Span, &mut E::State) -> E::Error,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
    where
        Self: Sized,
    {
        let start = inp.cursor();
        let old_alt = inp.take_alt();
        let res = self.parser.go::<M>(inp);

        if res.is_err() {
            // Can't fail!
            let mut new_alt = inp.take_alt().unwrap();
            let span = inp.span_since(&start);
            new_alt.err = (self.mapper)(new_alt.err, span, inp.state());

            inp.errors.alt = old_alt;
            inp.add_alt_err(&new_alt.pos, new_alt.err);
        }

        res
    }

    go_extra!(Self::Output);
}

/// See [`UnitParser::validate`]
pub struct Validate<A, F, O> {
    pub(crate) parser: A,
    pub(crate) validator: F,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, F: Copy, O> Copy for Validate<A, F, O> {}
impl<A: Clone, F: Clone, O> Clone for Validate<A, F, O> {
    fn clone(&self) -> Self {
        Validate {
            parser: self.parser.clone(),
            validator: self.validator.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, F, O> UnitParser<'src, I, E> for Validate<A, F, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E>,
    F: Fn(A::Output, &mut MapExtra<'src, '_, I, E>, &mut Emitter<E::Error>) -> O,
{
    type Output = O;

    #[inline(always)]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
    where
        Self: Sized,
    {
        let before = inp.cursor();
        let out = self.parser.go::<Emit>(inp)?;

        let mut emitter = Emitter::new();
        let out = (self.validator)(out, &mut MapExtra::new(&before, inp), &mut emitter);
        for err in emitter.errors() {
            inp.emit(before.clone(), err);
        }
        Ok(M::bind(|| out))
    }

    go_extra!(Self::Output);
}

// /// See [`UnitParser::or_else`].
// #[derive(Copy, Clone)]
// pub struct OrElse<A, F> {
//     pub(crate) parser: A,
//     pub(crate) or_else: F,
// }

// impl<'src, I, E, A, F> Parser<'src, I, E> for OrElse<A, F>
// where
//     I: Input<'src>,
//     E: ParserExtra<'src, I>,
//     A: Parser<'src, I, E>,
//     F: Fn(E::Error) -> Result<A::Output, E::Error>,
// {
//     type Output = A::Output;
//
//     #[inline(always)]
//     fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
//     where
//         Self: Sized,
//     {
//         let before = inp.save();
//         match self.parser.go::<M>(inp) {
//             Ok(out) => Ok(out),
//             Err(()) => {
//                 let err = inp.take_alt();
//                 match (self.or_else)(err.err) {
//                     Ok(out) => {
//                         inp.rewind(before);
//                         Ok(M::bind(|| out))
//                     }
//                     Err(new_err) => {
//                         inp.errors.alt = Some(Located {
//                             pos: err.pos,
//                             err: new_err,
//                         });
//                         Err(())
//                     }
//                 }
//             }
//         }
//     }

//     go_extra!(Self::Output);
// }

/// See [`UnitParser::contextual`].
pub struct Contextual<A, O> {
    pub(crate) inner: A,
    #[allow(dead_code)]
    pub(crate) phantom: EmptyPhantom<O>,
}

impl<A: Copy, O> Copy for Contextual<A, O> {}
impl<A: Clone, O> Clone for Contextual<A, O> {
    fn clone(&self) -> Self {
        Contextual {
            inner: self.inner.clone(),
            phantom: EmptyPhantom::new(),
        }
    }
}

impl<'src, I, E, A, O> UnitParser<'src, I, E> for Contextual<A, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
{
    type Output = A::Output;

    #[inline]
    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output> {
        Self::go_cfg::<M>(self, inp, true)
    }

    go_extra!(Self::Output);
}

impl<'src, I, E, A, O> ConfigParser<'src, I, E> for Contextual<A, O>
where
    I: Input<'src>,
    E: ParserExtra<'src, I>,
    A: UnitParser<'src, I, E, Output = O>,
{
    type Config = bool;

    #[inline]
    fn go_cfg<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        cfg: Self::Config,
    ) -> PResult<M, Self::Output> {
        let before = inp.cursor();
        if cfg {
            self.inner.go::<M>(inp)
        } else {
            let err_span = inp.span_since(&before);
            inp.add_alt([DefaultExpected::SomethingElse], None, err_span);
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn separated_by_at_least() {
        let parser = just::<_, _, extra::Default>('-')
            .separated_by(just(','))
            .at_least(3)
            .collect();

        assert_eq!(parser.parse("-,-,-").into_result(), Ok(vec!['-', '-', '-']));
    }

    #[test]
    fn separated_by_at_least_without_leading() {
        let parser = just::<_, _, extra::Default>('-')
            .separated_by(just(','))
            .at_least(3)
            .collect::<Vec<_>>();

        // Is empty means no errors
        assert!(parser.parse(",-,-,-").has_errors());
    }

    #[test]
    fn separated_by_at_least_without_trailing() {
        let parser = just::<_, _, extra::Default>('-')
            .separated_by(just(','))
            .at_least(3)
            .collect::<Vec<_>>();

        // Is empty means no errors
        assert!(parser.parse("-,-,-,").has_errors());
    }

    #[test]
    fn separated_by_at_least_with_leading() {
        let parser = just::<_, _, extra::Default>('-')
            .separated_by(just(','))
            .allow_leading()
            .at_least(3)
            .collect();

        assert_eq!(
            parser.parse(",-,-,-").into_result(),
            Ok(vec!['-', '-', '-'])
        );
        assert!(parser.parse(",-,-").has_errors());
    }

    #[test]
    fn separated_by_at_least_with_trailing() {
        let parser = just::<_, _, extra::Default>('-')
            .separated_by(just(','))
            .allow_trailing()
            .at_least(3)
            .collect();

        assert_eq!(
            parser.parse("-,-,-,").into_result(),
            Ok(vec!['-', '-', '-'])
        );
        assert!(parser.parse("-,-,").has_errors());
    }

    #[test]
    fn separated_by_leaves_last_separator() {
        let parser = just::<_, _, extra::Default>('-')
            .separated_by(just(','))
            .collect::<Vec<_>>()
            .then(just(','));
        assert_eq!(
            parser.parse("-,-,-,").into_result(),
            Ok((vec!['-', '-', '-'], ',')),
        )
    }
}

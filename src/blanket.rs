use super::*;

impl<'src, T, I, E> UnitParser<'src, I, E> for &T
where
    T: ?Sized + UnitParser<'src, I, E>,
    I: Input<'src>,
    E: ParserExtra<'src, I>,
{
    type Output = T::Output;

    fn go<M: Mode>(&self, inp: &mut InputRef<'src, '_, I, E>) -> PResult<M, Self::Output>
    where
        Self: Sized,
    {
        M::invoke(*self, inp)
    }

    go_extra!(Self::Output);
}

impl<'src, T, I, E> ConfigParser<'src, I, E> for &T
where
    T: ?Sized + ConfigParser<'src, I, E>,
    I: Input<'src>,
    E: ParserExtra<'src, I>,
{
    type Config = T::Config;

    fn go_cfg<M: Mode>(
        &self,
        inp: &mut InputRef<'src, '_, I, E>,
        cfg: Self::Config,
    ) -> PResult<M, Self::Output> {
        M::invoke_cfg(*self, inp, cfg)
    }
}

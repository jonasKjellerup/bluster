/// Implements the consumption of a specified `Builder` type.
/// This is used for `ChainedBuilder` when the chain is completed.
pub trait ConsumeBuilder<Builder> {
    fn consume(self, target: Builder) -> Self;
}

/// This structure facilitates chaining of builder types.
/// The structure dereferences to the generic `Builder` type.
pub struct ChainedBuilder<Builder, Result>
where Result: ConsumeBuilder<Builder>
{
    builder: Builder,
    result: Result,
}

impl<Builder, Result> ChainedBuilder<Builder, Result>
where Result: ConsumeBuilder<Builder>
{
    pub fn new(builder: Builder, result: Result) -> Self {
        ChainedBuilder {
            builder,
            result,
        }
    }

    pub fn complete_chain(mut self) -> Result {
        self.result.consume(self.builder)
    }
}

impl<Builder, Result> std::ops::Deref for ChainedBuilder<Builder, Result>
where Result: ConsumeBuilder<Builder>
{
    type Target = Builder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl<Builder, Result> std::ops::DerefMut for ChainedBuilder<Builder, Result>
where Result: ConsumeBuilder<Builder>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}

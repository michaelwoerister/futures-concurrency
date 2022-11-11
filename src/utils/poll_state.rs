use std::ops::{Deref, DerefMut};

/// Enumerate the current poll state.
#[derive(Debug, Clone, Copy, Default)]
#[repr(u8)]
pub(crate) enum PollState {
    /// Polling the underlying future.
    #[default]
    Pending,
    /// Data has been written to the output structure
    /// and the future should no longer be polled.
    Done,
    /// Data has been consumed from the output structure,
    /// and we should no longer reason about it.
    Consumed,
}

impl PollState {
    /// Returns `true` if the metadata is [`Pending`].
    ///
    /// [`Pending`]: Metadata::Pending
    #[must_use]
    pub(crate) fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Returns `true` if the poll state is [`Done`].
    ///
    /// [`Done`]: PollState::Done
    #[must_use]
    pub(crate) fn is_done(&self) -> bool {
        matches!(self, Self::Done)
    }

    /// Returns `true` if the poll state is [`Consumed`].
    ///
    /// [`Consumed`]: PollState::Consumed
    #[must_use]
    pub(crate) fn is_consumed(&self) -> bool {
        matches!(self, Self::Consumed)
    }
}

const MAX_INLINE_ENTRIES: usize = std::mem::size_of::<usize>() * 3 - 2;

pub(crate) enum PollStates {
    Inline(u8, [PollState; MAX_INLINE_ENTRIES]),
    Boxed(Box<[PollState]>),
}

impl PollStates {
    pub(crate) fn new(len: usize) -> Self {
        if len <= MAX_INLINE_ENTRIES {
            Self::Inline(len as u8, Default::default())
        } else {
            let mut states = Vec::new();
            debug_assert_eq!(states.capacity(), 0);
            states.reserve_exact(len);
            debug_assert_eq!(states.capacity(), len);
            states.resize(len, PollState::default());
            debug_assert_eq!(states.capacity(), len);
            Self::Boxed(states.into_boxed_slice())
        }
    }
}

impl Deref for PollStates {
    type Target = [PollState];

    fn deref(&self) -> &Self::Target {
        match self {
            PollStates::Inline(len, states) => &states[..*len as usize],
            Self::Boxed(states) => &states[..],
        }
    }
}

impl DerefMut for PollStates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            PollStates::Inline(len, states) => &mut states[..*len as usize],
            Self::Boxed(states) => &mut states[..],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PollStates, MAX_INLINE_ENTRIES};

    #[test]
    fn type_size() {
        assert_eq!(
            std::mem::size_of::<PollStates>(),
            std::mem::size_of::<usize>() * 3
        );
    }

    #[test]
    fn boxed_does_not_allocate_twice() {
        // Make sure the debug_assertions in PollStates::new() don't fail.
        let _ = PollStates::new(MAX_INLINE_ENTRIES + 10);
    }
}

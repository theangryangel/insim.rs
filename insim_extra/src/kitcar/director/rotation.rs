//! [`Rotation`]: where combos come from, and [`Cycle`], a list of them.

use std::future::Future;

use futures::{
    FutureExt,
    future::{self, BoxFuture},
};

use super::Combo;
#[cfg(doc)]
use super::{RaceDirector, RaceDirectorStatus};
use crate::world::World;

/// Why a [`Rotation`] could not produce a combo.
pub type RotationError = Box<dyn std::error::Error + Send + Sync>;

/// Picks the next combo: once at start, then each time the [`RaceDirector`]
/// moves on.
///
/// `next` runs inside the director's update, so it must not do I/O. Read what
/// you need from `current` and `world`, then do slow work (loading a `.lyt`,
/// querying a database) in the returned future. If the future fails, the
/// director stays on the current combo.
///
/// Implemented for:
/// - closures `FnMut(Option<&Combo>, &World) -> impl Future<Output =
///   Result<Combo, RotationError>>`;
/// - [`Cycle`], a list in order or reshuffled each pass;
/// - [`Combo`], which always returns itself.
///
/// ```ignore
/// RaceDirector::new(|_: Option<&Combo>, world: &World| {
///     let busy = world.connection_count() > 10;
///     async move {
///         let path = if busy { "big.lyt" } else { "small.lyt" };
///         let lyt = insim_lyt::Lyt::from_path(path)?;
///         let combo = Combo::untimed("FE1X".parse()?).layout(Layout::Objects(lyt.objects));
///         // Rust can't infer an async block's error type, so name it once.
///         Ok::<_, RotationError>(combo)
///     }
/// })
/// ```
pub trait Rotation: Send + 'static {
    /// The next combo. `current` is `None` when the director starts.
    fn next(
        &mut self,
        current: Option<&Combo>,
        world: &World,
    ) -> BoxFuture<'static, Result<Combo, RotationError>>;

    /// The combo `next` will produce, if known without asking. Shown to
    /// players as what's coming up. `None` by default, as closures can't tell.
    fn peek(&self) -> Option<&Combo> {
        None
    }
}

impl<F, Fut> Rotation for F
where
    F: FnMut(Option<&Combo>, &World) -> Fut + Send + 'static,
    Fut: Future<Output = Result<Combo, RotationError>> + Send + 'static,
{
    fn next(
        &mut self,
        current: Option<&Combo>,
        world: &World,
    ) -> BoxFuture<'static, Result<Combo, RotationError>> {
        self(current, world).boxed()
    }
}

/// A rotation that never changes.
impl Rotation for Combo {
    fn next(
        &mut self,
        _: Option<&Combo>,
        _: &World,
    ) -> BoxFuture<'static, Result<Combo, RotationError>> {
        future::ready(Ok(self.clone())).boxed()
    }

    fn peek(&self) -> Option<&Combo> {
        Some(self)
    }
}

/// A list of combos, looping after the last.
///
/// The first pass runs in the order given. With [`shuffle`](Self::shuffle),
/// each later pass is a fresh random order that never opens with the combo
/// that just played. The order is drawn as a pass ends, so
/// [`RaceDirectorStatus::next`] always names the real next combo.
#[derive(Debug, Clone)]
pub struct Cycle {
    combos: Vec<Combo>,
    /// Indices into `combos`, in this pass's order.
    order: Vec<usize>,
    /// Position in `order`.
    next: usize,
    shuffle: Option<fastrand::Rng>,
}

impl Cycle {
    /// Cycle through `combos`, in order.
    ///
    /// # Panics
    ///
    /// If `combos` is empty.
    pub fn new(combos: impl IntoIterator<Item = Combo>) -> Self {
        let combos: Vec<Combo> = combos.into_iter().collect();
        assert!(!combos.is_empty(), "a cycle needs at least one combo");
        Self {
            order: (0..combos.len()).collect(),
            combos,
            next: 0,
            shuffle: None,
        }
    }

    /// Reshuffle at the end of each pass.
    pub fn shuffle(self) -> Self {
        self.shuffle_with(fastrand::Rng::new())
    }

    /// Like [`shuffle`](Self::shuffle), from a seed, for repeatable orders.
    pub fn shuffle_seeded(self, seed: u64) -> Self {
        self.shuffle_with(fastrand::Rng::with_seed(seed))
    }

    fn shuffle_with(mut self, rng: fastrand::Rng) -> Self {
        self.shuffle = Some(rng);
        self
    }

    /// Start the next pass, reshuffled if asked, never opening with `last`.
    fn next_pass(&mut self, last: usize) {
        self.next = 0;
        let Some(rng) = &mut self.shuffle else {
            return;
        };
        rng.shuffle(&mut self.order);
        if self.order.len() > 1 && self.order[0] == last {
            let swap = rng.usize(1..self.order.len());
            self.order.swap(0, swap);
        }
    }
}

impl Rotation for Cycle {
    fn next(
        &mut self,
        _: Option<&Combo>,
        _: &World,
    ) -> BoxFuture<'static, Result<Combo, RotationError>> {
        let index = self.order[self.next];
        self.next += 1;
        if self.next == self.order.len() {
            self.next_pass(index);
        }
        future::ready(Ok(self.combos[index].clone())).boxed()
    }

    fn peek(&self) -> Option<&Combo> {
        self.combos.get(self.order[self.next])
    }
}

#[cfg(test)]
mod tests {
    use insim::core::track::Track;

    use super::*;
    use crate::kitcar::director::tests::{as1, bl1, fe1x};

    /// The tracks `cycle` produces over `count` combos, checking each against
    /// what it said was next.
    fn drawn(cycle: &mut Cycle, count: usize) -> Vec<Track> {
        let world = World::new();
        (0..count)
            .map(|_| {
                let peeked = cycle.peek().map(|combo| combo.track);
                let combo = cycle
                    .next(None, &world)
                    .now_or_never()
                    .expect("a cycle is ready at once")
                    .expect("a cycle never fails");
                assert_eq!(peeked, Some(combo.track), "peek named the next combo");
                combo.track
            })
            .collect()
    }

    #[test]
    fn a_shuffled_cycle_plays_each_once_per_pass_and_never_twice_running() {
        let tracks = [fe1x(), bl1(), as1()];
        for seed in 0..50 {
            let mut cycle = Cycle::new(tracks.map(Combo::untimed)).shuffle_seeded(seed);
            let drawn = drawn(&mut cycle, tracks.len() * 10);

            // The first pass is in the order given.
            assert_eq!(drawn[..3], tracks, "seed {seed}");
            for pass in drawn.chunks(tracks.len()) {
                let mut sorted = pass.to_vec();
                sorted.sort_by_key(ToString::to_string);
                let mut all = tracks.to_vec();
                all.sort_by_key(ToString::to_string);
                assert_eq!(sorted, all, "seed {seed}: every track once a pass");
            }
            assert!(
                drawn.windows(2).all(|pair| pair[0] != pair[1]),
                "seed {seed}: {drawn:?}"
            );
        }
    }

    #[test]
    fn a_cycle_of_one_repeats_it_even_shuffled() {
        let mut cycle = Cycle::new([Combo::untimed(fe1x())]).shuffle_seeded(7);
        assert_eq!(drawn(&mut cycle, 3), [fe1x(); 3]);
    }
}

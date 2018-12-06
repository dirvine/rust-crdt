//! The `vclock` crate provides a generic vector clock implementation.
//!
//! # Examples
//!
//! ```
//! use crdts::*;
//! let mut a = VClock::new();
//! let mut b = VClock::new();
//! a.apply_dot(Dot::new("A".to_string(), 2));
//! b.apply_dot(Dot::new("A".to_string(), 1));
//! assert!(a > b);
//! ```

// TODO: we have a mixture of language here with witness and actor. Clean this up

use super::*;

use std::fmt::{self, Debug, Display};
use std::cmp::{self, Ordering};
use std::collections::{BTreeMap, btree_map};
use std::hash::Hash;

/// Common Actor type. Actors are unique identifier for every `thing` mutating a VClock.
/// VClock based CRDT's will need to expose this Actor type to the user.
pub trait Actor: Ord + Clone + Hash + Debug {}
impl<A: Ord + Clone + Hash + Debug> Actor for A {}


/// Dot is a version marker for a single actor
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dot<A> {
    /// The actor identifier
    pub actor: A,
    /// The current version of this actor
    pub counter: u64
}

impl<A: Actor> Dot<A> {
    /// Build a Dot from an actor and counter
    pub fn new(actor: A, counter: u64) -> Self {
        Dot { actor, counter }
    }
}

/// A `VClock` is a standard vector clock.
/// It contains a set of "actors" and associated counters.
/// When a particular actor witnesses a mutation, their associated
/// counter in a `VClock` is incremented. `VClock` is typically used
/// as metadata for associated application data, rather than as the
/// container for application data. `VClock` just tracks causality.
/// It can tell you if something causally descends something else,
/// or if different replicas are "concurrent" (were mutated in
/// isolation, and need to be resolved externally).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VClock<A: Actor> {
    /// dots is the mapping from actors to their associated counters
    pub dots: BTreeMap<A, u64>,
}

impl<A: Actor> Default for VClock<A> {
    fn default() -> Self {
        VClock::new()
    }
}

impl<A: Actor> PartialOrd for VClock<A> {
    fn partial_cmp(&self, other: &VClock<A>) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if other.dots.iter().all(|(w, c)| self.get(w) >= *c) {
            Some(Ordering::Greater)
        } else if self.dots.iter().all(|(w, c)| other.get(w) >= *c) {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

impl<A: Actor + Display> Display for VClock<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<")?;
        for (i, (actor, count)) in self.dots.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}:{}", actor, count)?;
        }
        write!(f, ">")
    }
}

impl<A: Actor> Causal<A> for VClock<A> {
    /// Truncates to the greatest-lower-bound of the given VClock and self
    /// ``` rust
    /// use crdts::*;
    /// let mut c = VClock::new();
    /// c.apply_dot(Dot::new(23, 6));
    /// c.apply_dot(Dot::new(89, 14));
    /// let c2 = c.clone();
    ///
    /// c.truncate(&c2); // should be a no-op
    /// assert_eq!(c, c2);
    ///
    /// c.apply_dot(Dot::new(43, 1));
    /// assert_eq!(c.get(&43), 1);
    /// c.truncate(&c2); // should remove the 43 => 1 entry
    /// assert_eq!(c.get(&43), 0);
    /// ```
    fn truncate(&mut self, other: &VClock<A>) {
        let mut actors_to_remove: Vec<A> = Vec::new();
        for (actor, count) in self.dots.iter_mut() {
            let min_count = cmp::min(*count, other.get(actor));
            if min_count > 0 {
                *count = min_count
            } else {
                // Since an actor missing from the dots map has an implied counter of 0
                // we can save some memory, and remove the actor.
                actors_to_remove.push(actor.clone())
            }
        }

        // finally, remove all the zero counter actor
        for actor in actors_to_remove {
            self.dots.remove(&actor);
        }
    }
}

impl<A: Actor> CmRDT for VClock<A> {
    type Op = Dot<A>;

    fn apply(&mut self, dot: &Self::Op) {
        self.apply_dot(dot.clone());
    }
}

impl<A: Actor> CvRDT for VClock<A> {
    fn merge(&mut self, other: &VClock<A>) {
        for (actor, counter) in other.dots.iter() {
            self.apply_dot(Dot::new(actor.clone(), *counter));
        }
    }
}

impl<A: Actor> VClock<A> {
    /// Returns a new `VClock` instance.
    pub fn new() -> Self {
        VClock { dots: BTreeMap::new() }
    }

    /// Apply a Dot to this vclock.
    ///
    /// # Examples
    /// ```
    /// use crdts::*;
    /// let mut v = VClock::new();
    ///
    /// v.apply_dot(Dot::new("A".to_string(), 2));
    /// 
    /// // now all dots applied to `v` from actor `A` where
    /// // the counter is not bigger than 2 are nops.
    /// v.apply_dot(Dot::new("A".to_string(), 0));
    /// assert_eq!(v.get(&"A".to_string()), 2);
    /// ```
    pub fn apply_dot(&mut self, dot: Dot<A>) {
        if self.get(&dot.actor) < dot.counter {
            self.dots.insert(dot.actor, dot.counter);
        }
    }

    /// Immediately increment an actor's counter.
    ///
    /// # Examples
    /// ```
    /// use crdts::VClock;
    /// let mut v = VClock::new();
    ///
    /// v.apply_inc("A".to_string());
    /// assert_eq!(v.get(&"A".to_string()), 1);
    /// ```
    pub fn apply_inc(&mut self, actor: A) {
        let inc_counter = self.get(&actor) + 1;
        self.apply_dot(Dot::new(actor, inc_counter));
    }

    /// Generate Op to increment an actor's counter.
    ///
    /// # Examples
    /// ```
    /// use crdts::{VClock, CmRDT};
    /// let mut a = VClock::new();
    ///
    /// // `a.inc()` does not mutate the vclock!
    /// let op = a.inc("A".to_string());
    /// assert_eq!(a, VClock::new());
    ///
    /// // we must apply the op to the VClock to have
    /// // its edit take effect.
    /// a.apply(&op);
    /// assert_eq!(a.get(&"A".to_string()), 1);
    ///
    /// // Op's can be replicated to another node and
    /// // applied to the local state there.
    /// let mut other_node = VClock::new();
    /// other_node.apply(&op);
    /// assert_eq!(other_node.get(&"A".to_string()), 1);
    /// ```
    pub fn inc(&self, actor: A) -> Dot<A> {
        let next = self.get(&actor) + 1;
        Dot { actor, counter: next }
    }

    /// True if two vector clocks have diverged.
    ///
    /// # Examples
    /// ```
    /// use crdts::{VClock, CmRDT};
    /// let (mut a, mut b) = (VClock::new(), VClock::new());
    /// let a_op = a.inc("A".to_string());
    /// a.apply(&a_op);
    /// let b_op = b.inc("B".to_string());
    /// b.apply(&b_op);
    /// assert!(a.concurrent(&b));
    /// ```
    pub fn concurrent(&self, other: &VClock<A>) -> bool {
        self.partial_cmp(other).is_none()
    }

    /// Return the associated counter for this actor.
    /// All actors not in the vclock have an implied count of 0
    pub fn get(&self, actor: &A) -> u64 {
        self.dots.get(actor)
            .cloned()
            .unwrap_or(0)
    }

    /// Returns `true` if this vector clock contains nothing.
    pub fn is_empty(&self) -> bool {
        self.dots.is_empty()
    }

    /// Returns the common elements (same actor and counter)
    /// for two `VClock` instances.
    pub fn intersection(&self, other: &VClock<A>) -> VClock<A> {
        let mut dots = BTreeMap::new();
        for (actor, counter) in self.dots.iter() {
            let other_counter = other.get(actor);
            if other_counter == *counter {
                dots.insert(actor.clone(), *counter);
            }
        }
        VClock { dots }
    }

    /// Returns an iterator over the dots in this vclock
    pub fn iter(&self) -> impl Iterator<Item=Dot<&A>> {
        self.dots
            .iter()
            .map(|(a, c)| Dot { actor: a, counter: *c })
    }

    /// Forget actors who appear in the given VClock with descendent dots
    pub fn subtract(&mut self, other: &VClock<A>) {
        for Dot { actor, counter } in other.iter() {
            if counter >= self.get(&actor) {
                self.dots.remove(&actor);
            }
        }
    }
}

/// Generated from calls to VClock::into_iter()
pub struct IntoIter<A: Actor> {
    btree_iter: btree_map::IntoIter<A, u64>
}

impl<A: Actor> std::iter::Iterator for IntoIter<A> {
    type Item = Dot<A>;

    fn next(&mut self) -> Option<Dot<A>> {
        self.btree_iter
            .next()
            .map(|(actor, counter)| Dot::new(actor, counter))
    }
}

impl<A: Actor> std::iter::IntoIterator for VClock<A> {
    type Item = Dot<A>;
    type IntoIter = IntoIter<A>;
    
    /// Consumes the vclock and returns an iterator over dots in the clock
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            btree_iter: self.dots.into_iter()
        }
    }
}

impl<A: Actor> std::iter::FromIterator<Dot<A>> for VClock<A> {
    fn from_iter<I: IntoIterator<Item=Dot<A>>>(iter: I) -> Self {
        let mut clock = VClock::new();

        for dot in iter {
            clock.apply(&dot);
        }

        clock
    }
}

impl<A: Actor> From<Dot<A>> for VClock<A> {
    fn from(dot: Dot<A>) -> Self {
        let mut clock = VClock::new();
        clock.apply(&dot);
        clock
    }
}

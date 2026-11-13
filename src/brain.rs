//! What the kitten is doing: one activity at a time. The status line only sets the mood it WANTS; the brain switches
//! when the paws are free, runs the mood's own loop on a clock that stops while a rare event (a sip, a stretch...)
//! has the paws, and picks those events now and then at moments where the loop can pause.

use crate::scene::Mood;

/// A rare break from the mood's loop.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    Sip,
    /// the mug turns out to be empty: kettle, pour, then a sip of the fresh coffee
    Refill,
    Stretch,
    Yawn,
    /// glances off to one side of the picture, then the other
    LookAround,
}

impl Event {
    pub fn secs(self) -> f64 {
        match self {
            Event::Sip => 2.4,
            Event::Refill => 7.0,
            Event::Stretch => 2.6,
            Event::Yawn => 2.2,
            Event::LookAround => 3.2,
        }
    }
}

/// How often each event comes up (relative weights).
const EVENTS: [(Event, f64); 5] =
    [(Event::Sip, 0.45), (Event::Refill, 0.15), (Event::Stretch, 0.15), (Event::Yawn, 0.10), (Event::LookAround, 0.15)];
/// Seconds between events: at least the first, plus up to the second at random.
const GAP: (f64, f64) = (15.0, 20.0);

/// One frame's answer: the mood whose loop shows, how far that loop has run, and the event if one has the paws.
#[derive(Clone, Copy, Debug)]
pub struct Act {
    /// None while napping
    pub mood: Option<Mood>,
    /// seconds the mood's loop has run since the status line started (stops during events)
    pub work: f64,
    /// a new number per status line, for anything the loop picks at random per line
    pub line: u64,
    pub event: Option<(Event, f32)>,
}

pub struct Brain {
    wanted: Option<Mood>,
    mood: Option<Mood>,
    work: f64,
    line: u64,
    /// a new status line came in: its loop clock (its age so far) takes over when it shows
    restart: Option<f64>,
    event: Option<(Event, f64)>,
    pub next_event: f64,
    last_t: f64,
}

impl Brain {
    pub fn new() -> Self {
        Brain { wanted: None, mood: None, work: 0.0, line: 0, restart: None, event: None, next_event: 10.0, last_t: 0.0 }
    }

    /// The status line asks for `mood` (None = idle); `new_line` when its text changed, so the loop starts over.
    pub fn want(&mut self, mood: Option<Mood>, new_line: bool, age: f64) {
        self.wanted = mood;
        if new_line {
            self.restart = Some(age.max(0.0));
            self.line = self.line.wrapping_add(1);
        }
    }

    /// Start an event now (snapshots and tests).
    pub fn force(&mut self, e: Event, start: f64) {
        self.event = Some((e, start));
        self.mood = self.mood.or(self.wanted);
    }

    /// The loop's own round lengths, so events wait for a moment where it can pause.
    fn safe(mood: Mood, work: f64, rounds: &dyn Fn(Mood) -> Option<(f64, f64, f64)>) -> bool {
        match rounds(mood) {
            // only in the part of a round from `from` to `to` (a test round's start, a finished drawing)
            Some((round, from, to)) => (from..to).contains(&(work.max(0.0) % round)),
            None => true,
        }
    }

    /// One frame. `rand` gives 0..1; `rounds` says for a mood (round length, and the part of
    /// a round where it can pause: from, to).
    pub fn tick(&mut self, t: f64, rand: &mut dyn FnMut() -> f64, rounds: &dyn Fn(Mood) -> Option<(f64, f64, f64)>) -> Act {
        let dt = (t - self.last_t).clamp(0.0, 0.2);
        self.last_t = t;
        // sleep ends anything; otherwise a running event finishes first (a refill ends with a sip)
        if self.wanted.is_none() {
            self.event = None;
        }
        if let Some((e, start)) = self.event {
            if t - start >= e.secs() {
                self.event = (e == Event::Refill).then_some((Event::Sip, start + e.secs()));
                self.next_event = self.next_event.max(t + GAP.0);
            }
        }
        if self.event.is_none() {
            self.mood = self.wanted;
            if let Some(w) = self.restart.take() {
                self.work = w;
            } else if self.mood.is_some() {
                self.work += dt;
            }
            if let Some(m) = self.mood {
                if t > self.next_event && Self::safe(m, self.work, rounds) {
                    let mut pick = rand() * EVENTS.iter().map(|e| e.1).sum::<f64>();
                    let e = EVENTS.iter().find(|e| {
                        pick -= e.1;
                        pick < 0.0
                    });
                    self.event = Some((e.map_or(Event::Sip, |e| e.0), t));
                    self.next_event = t + GAP.0 + rand() * GAP.1;
                }
            }
        }
        Act {
            mood: self.mood,
            work: self.work,
            line: self.line,
            event: self.event.map(|(e, s)| (e, (((t - s) / e.secs()) as f32).clamp(0.0, 0.999))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rounds(m: Mood) -> Option<(f64, f64, f64)> {
        (m == Mood::Thinking).then_some((10.0, 6.0, 10.0))
    }

    #[test]
    fn a_mood_change_waits_for_the_event_and_the_clock_stops_meanwhile() {
        let mut b = Brain::new();
        let mut r = || 0.5;
        b.want(Some(Mood::Busy), true, 0.0);
        b.tick(0.0, &mut r, &rounds);
        b.tick(1.0, &mut r, &rounds);
        b.force(Event::Refill, 1.0);
        let w = b.tick(1.1, &mut r, &rounds).work;
        b.want(Some(Mood::Waiting), true, 0.0);
        let a = b.tick(5.0, &mut r, &rounds);
        assert_eq!(a.mood, Some(Mood::Busy));
        assert_eq!(a.work, w);
        assert_eq!(a.event.map(|e| e.0), Some(Event::Refill));
        // refill (7 s) then its sip (2.4 s)
        assert_eq!(b.tick(8.5, &mut r, &rounds).event.map(|e| e.0), Some(Event::Sip));
        let a = b.tick(10.5, &mut r, &rounds);
        assert_eq!((a.mood, a.event), (Some(Mood::Waiting), None));
        assert_eq!(a.work, 0.0);
    }

    #[test]
    fn events_wait_for_the_loops_safe_part() {
        let mut b = Brain::new();
        let mut r = || 0.0;
        b.want(Some(Mood::Thinking), true, 0.0);
        b.next_event = 0.0;
        let mut t = 0.0;
        let mut first = None;
        while t < 20.0 {
            t += 0.05;
            let a = b.tick(t, &mut r, &rounds);
            if first.is_none() && a.event.is_some() {
                first = Some(a.work % 10.0);
            }
        }
        assert!(first.unwrap() >= 6.0, "{first:?}");
    }

    #[test]
    fn sleep_ends_an_event() {
        let mut b = Brain::new();
        let mut r = || 0.5;
        b.want(Some(Mood::Busy), true, 0.0);
        b.force(Event::Stretch, 0.0);
        b.tick(0.5, &mut r, &rounds);
        b.want(None, false, 0.0);
        let a = b.tick(0.6, &mut r, &rounds);
        assert_eq!((a.mood, a.event), (None, None));
    }
}

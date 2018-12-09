const SECS_PER_MINUTE: u32 = 60;
const SECS_PER_HOUR: u32 = 3600;

// these config variables are const right now, but should move into a struct
const DEFAULT_WORK_MIN: u32 = 25;
const SHORT_BREAK_MIN: u32 = 5;
const LONG_BREAK_MIN: u32 = 15;

const DEFAULT_WORK_SECS: u32 = DEFAULT_WORK_MIN * SECS_PER_MINUTE;
const SHORT_BREAK_SECS: u32 = SHORT_BREAK_MIN * SECS_PER_MINUTE;
const LONG_BREAK_SECS: u32 = LONG_BREAK_MIN * SECS_PER_MINUTE;

/// Formats seconds as HH:MM:SS
///
/// ```
/// let formatted = String::from("1:02:33");
/// let secs = 3753;
///
/// assert_eq!(pmdr::format_secs(secs), formatted);
/// ```
pub fn format_secs(secs: u32) -> String {
    let hours = secs / SECS_PER_HOUR;
    let minutes = (secs - SECS_PER_HOUR * hours) / SECS_PER_MINUTE;
    let seconds = secs % SECS_PER_MINUTE;

    format!("{}:{:02}:{:02}", hours, minutes, seconds)
}

struct WorkTimer {
    count: usize,
    timer: u32,
}
struct Paused(Box<TimerState>);
struct BreakTimer {
    count: usize,
    timer: u32,
}
struct Stopped {
    count: usize,
}

impl WorkTimer {
    fn new(count: usize, timer: u32) -> Box<WorkTimer> {
        Box::new(WorkTimer { count, timer })
    }
}

impl BreakTimer {
    fn new(count: usize, timer: u32) -> Box<BreakTimer> {
        Box::new(BreakTimer { count, timer })
    }
}

trait TimerState {
    fn count(&self) -> usize;

    fn timer(&self) -> u32;

    fn ticking(&self) -> bool {
        true
    }

    fn on_break(&self) -> bool {
        false
    }

    fn label(&self) -> String;

    fn toggle(self: Box<Self>) -> Box<dyn TimerState>;

    fn tick(self: Box<Self>) -> (Box<dyn TimerState>, bool);

    fn stop(self: Box<Self>) -> Box<dyn TimerState> {
        let stopped = Stopped {
            count: self.count(),
        };
        Box::new(stopped)
    }
}

impl TimerState for Stopped {
    fn count(&self) -> usize {
        self.count
    }

    fn timer(&self) -> u32 {
        DEFAULT_WORK_SECS
    }

    fn ticking(&self) -> bool {
        false
    }

    fn label(&self) -> String {
        format!("Stopped")
    }

    fn toggle(self: Box<Self>) -> Box<dyn TimerState> {
        WorkTimer::new(self.count(), self.timer())
    }

    fn tick(self: Box<Self>) -> (Box<dyn TimerState>, bool) {
        (self, false)
    }

    // if stop is called on Stopped, we reset the tally.
    // this is kind of hacky
    fn stop(self: Box<Self>) -> Box<dyn TimerState> {
        Box::new(Stopped { count: 0 })
    }
}

impl TimerState for Paused {
    fn count(&self) -> usize {
        self.0.count()
    }

    fn timer(&self) -> u32 {
        self.0.timer()
    }

    fn ticking(&self) -> bool {
        false
    }

    fn label(&self) -> String {
        let contained = self.0.label();
        format!("Paused ({})", contained)
    }

    fn toggle(self: Box<Self>) -> Box<dyn TimerState> {
        self.0
    }

    fn tick(self: Box<Self>) -> (Box<dyn TimerState>, bool) {
        (self, false)
    }
}

impl TimerState for WorkTimer {
    fn count(&self) -> usize {
        self.count
    }

    fn timer(&self) -> u32 {
        self.timer
    }

    fn label(&self) -> String {
        format!("Keep Going!")
    }

    fn toggle(self: Box<Self>) -> Box<dyn TimerState> {
        Box::new(Paused(self))
    }

    fn tick(self: Box<Self>) -> (Box<dyn TimerState>, bool) {
        let count = self.count;
        let timer = self.timer - 1;

        let new_state: Box<dyn TimerState> = if timer <= 0 {
            let count = count + 1;
            let timer = if count % 4 == 0 {
                LONG_BREAK_SECS
            } else {
                SHORT_BREAK_SECS
            };

            BreakTimer::new(count, timer)
        } else {
            WorkTimer::new(count, timer)
        };

        (new_state, timer <= 0)
    }
}

impl TimerState for BreakTimer {
    fn count(&self) -> usize {
        self.count
    }

    fn timer(&self) -> u32 {
        self.timer
    }

    fn label(&self) -> String {
        format!("On Break")
    }

    fn on_break(&self) -> bool {
        true
    }

    fn toggle(self: Box<Self>) -> Box<dyn TimerState> {
        Box::new(Paused(self))
    }

    fn tick(self: Box<Self>) -> (Box<dyn TimerState>, bool) {
        let count = self.count;
        let timer = self.timer - 1;

        let new_state: Box<dyn TimerState> = if timer <= 0 {
            WorkTimer::new(count, DEFAULT_WORK_SECS)
        } else {
            BreakTimer::new(count, timer)
        };

        (new_state, timer <= 0)
    }
}

pub struct PMDRApp {
    timer_state: Option<Box<TimerState>>,
}

impl PMDRApp {
    pub fn new() -> PMDRApp {
        let timer_state: Box<dyn TimerState> = WorkTimer::new(0, DEFAULT_WORK_SECS);
        let timer_state = Some(timer_state);

        PMDRApp { timer_state }
    }

    /// Updates the timer and returns a boolean if the state changed
    pub fn tick(&mut self) -> bool {
        let (new_state, state_changed) = self.timer_state.take().unwrap().tick();

        self.timer_state = Some(new_state);

        state_changed
    }

    /// Gets the formatted string of the time remaining
    pub fn countdown_string(&self) -> String {
        let secs = self.timer_state.as_ref().map_or(0, |state| state.timer());

        format_secs(secs)
    }

    /// Toggle the app's timer between ticking and paused states
    pub fn toggle_timer(&mut self) -> bool {
        let new_state = self.timer_state.take().unwrap().toggle();
        let ticking = &new_state.ticking();
        self.timer_state = Some(new_state);
        *ticking
    }

    /// Gets the current tally
    pub fn tally(&self) -> usize {
        self.timer_state.as_ref().map_or(0, |state| state.count())
    }

    /// Stops the application, but keeps the tally
    pub fn stop(&mut self) {
        let new_state = self.timer_state.take().unwrap().stop();
        self.timer_state = Some(new_state);
    }

    /// Returns true if the timer is ticking
    pub fn ticking(&self) -> bool {
        self.timer_state
            .as_ref()
            .map_or(false, |state| state.ticking())
    }

    pub fn state_label(&self) -> String {
        self.timer_state
            .as_ref()
            .map_or_else(|| String::from(""), |state| state.label())
    }

    pub fn on_break(&self) -> bool {
        self.timer_state
            .as_ref()
            .map_or(false, |state| state.on_break())
    }
}

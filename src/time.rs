/*! Time structures.

The `time` module contains structures used to represent both
absolute and relative time.

 - [Instant] is used to represent absolute time.
 - [Duration] is used to represent relative time.

Both have microsecond resolution. Their accessors are named and behave like the
ones on [`core::time::Duration`]: `as_*` returns the whole value, `subsec_*`
returns only the part below one second.

[Instant]: struct.Instant.html
[Duration]: struct.Duration.html
*/

use core::{fmt, ops};

/// A representation of an absolute time value.
///
/// The `Instant` type is a wrapper around a `i64` value that
/// represents a number of microseconds, monotonically increasing
/// since an arbitrary moment in time, such as system startup.
///
/// * A value of `0` is inherently arbitrary.
/// * A value less than `0` indicates a time before the starting
///   point.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    micros: i64,
}

impl Instant {
    /// The starting point.
    pub const ZERO: Instant = Instant::from_micros(0);

    /// The earliest representable instant.
    pub const MIN: Instant = Instant::from_micros(i64::MIN);

    /// The latest representable instant.
    pub const MAX: Instant = Instant::from_micros(i64::MAX);

    /// Create a new `Instant` from a number of microseconds.
    pub const fn from_micros(micros: i64) -> Instant {
        Instant { micros }
    }

    /// Create a new `Instant` from a number of milliseconds.
    pub const fn from_millis(millis: i64) -> Instant {
        Instant { micros: millis * 1000 }
    }

    /// Create a new `Instant` from a number of seconds.
    pub const fn from_secs(secs: i64) -> Instant {
        Instant { micros: secs * 1000000 }
    }

    /// Create a new `Instant` from the current [std::time::SystemTime].
    ///
    /// Requires the `std` feature.
    ///
    /// See [std::time::SystemTime::now]
    ///
    /// [std::time::SystemTime]: https://doc.rust-lang.org/std/time/struct.SystemTime.html
    /// [std::time::SystemTime::now]: https://doc.rust-lang.org/std/time/struct.SystemTime.html#method.now
    #[cfg(feature = "std")]
    pub fn now() -> Instant {
        Self::from(::std::time::SystemTime::now())
    }

    /// The number of whole seconds since the starting point.
    ///
    /// Truncates towards zero for instants before the starting point.
    pub const fn as_secs(&self) -> i64 {
        self.micros / 1000000
    }

    /// The number of milliseconds since the starting point.
    pub const fn as_millis(&self) -> i64 {
        self.micros / 1000
    }

    /// The number of microseconds since the starting point.
    pub const fn as_micros(&self) -> i64 {
        self.micros
    }

    /// The number of milliseconds past [`as_secs`](Self::as_secs).
    ///
    /// Always less than 1000 in absolute value.
    pub const fn subsec_millis(&self) -> i64 {
        self.micros % 1000000 / 1000
    }

    /// The number of microseconds past [`as_secs`](Self::as_secs).
    ///
    /// Always less than 1000000 in absolute value.
    pub const fn subsec_micros(&self) -> i64 {
        self.micros % 1000000
    }

    /// The amount of time elapsed from `earlier` to this instant.
    ///
    /// Returns [`Duration::ZERO`] if `earlier` is later than this instant.
    pub const fn duration_since(&self, earlier: Instant) -> Duration {
        self.saturating_duration_since(earlier)
    }

    /// The amount of time elapsed from `earlier` to this instant.
    ///
    /// Returns `None` if `earlier` is later than this instant.
    pub const fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        if self.micros < earlier.micros {
            None
        } else {
            Some(Duration::from_micros(self.micros.abs_diff(earlier.micros)))
        }
    }

    /// The amount of time elapsed from `earlier` to this instant.
    ///
    /// Returns [`Duration::ZERO`] if `earlier` is later than this instant.
    pub const fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        if self.micros < earlier.micros {
            Duration::ZERO
        } else {
            Duration::from_micros(self.micros.abs_diff(earlier.micros))
        }
    }

    /// This instant plus `duration`, or `None` if the result does not fit.
    pub const fn checked_add(&self, duration: Duration) -> Option<Instant> {
        if duration.micros > i64::MAX as u64 {
            return None;
        }
        match self.micros.checked_add(duration.micros as i64) {
            Some(micros) => Some(Instant { micros }),
            None => None,
        }
    }

    /// This instant minus `duration`, or `None` if the result does not fit.
    pub const fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        if duration.micros > i64::MAX as u64 {
            return None;
        }
        match self.micros.checked_sub(duration.micros as i64) {
            Some(micros) => Some(Instant { micros }),
            None => None,
        }
    }

    /// The amount of time elapsed since this instant.
    ///
    /// Requires the `std` feature. Returns [`Duration::ZERO`] if this instant
    /// is in the future.
    #[cfg(feature = "std")]
    pub fn elapsed(&self) -> Duration {
        Instant::now().saturating_duration_since(*self)
    }
}

#[cfg(feature = "std")]
impl From<::std::time::Instant> for Instant {
    fn from(other: ::std::time::Instant) -> Instant {
        static REFERENTIAL: ::std::sync::LazyLock<::std::time::Instant> =
            ::std::sync::LazyLock::new(::std::time::Instant::now);

        let n = other.saturating_duration_since(*REFERENTIAL);
        Self::from_micros(n.as_secs() as i64 * 1000000 + n.subsec_micros() as i64)
    }
}

#[cfg(feature = "std")]
impl From<::std::time::SystemTime> for Instant {
    fn from(other: ::std::time::SystemTime) -> Instant {
        let n = other
            .duration_since(::std::time::UNIX_EPOCH)
            .expect("start time must not be before the unix epoch");
        Self::from_micros(n.as_secs() as i64 * 1000000 + n.subsec_micros() as i64)
    }
}

#[cfg(feature = "std")]
impl From<Instant> for ::std::time::SystemTime {
    fn from(val: Instant) -> Self {
        ::std::time::UNIX_EPOCH + ::std::time::Duration::from_micros(val.micros as u64)
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:0>3}s", self.as_secs(), self.subsec_millis())
    }
}

impl ops::Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Instant {
        Instant::from_micros(self.micros + rhs.as_micros() as i64)
    }
}

impl ops::AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        self.micros += rhs.as_micros() as i64;
    }
}

impl ops::Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Duration) -> Instant {
        Instant::from_micros(self.micros - rhs.as_micros() as i64)
    }
}

impl ops::SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        self.micros -= rhs.as_micros() as i64;
    }
}

impl ops::Sub<Instant> for Instant {
    type Output = Duration;

    /// Saturates to [`Duration::ZERO`] if `rhs` is later than `self`, like
    /// [`Instant::duration_since`].
    fn sub(self, rhs: Instant) -> Duration {
        self.saturating_duration_since(rhs)
    }
}

/// A relative amount of time.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    micros: u64,
}

impl Duration {
    /// A duration of zero time.
    pub const ZERO: Duration = Duration::from_micros(0);

    /// The longest possible duration we can encode.
    pub const MAX: Duration = Duration::from_micros(u64::MAX);

    /// Create a new `Duration` from a number of microseconds.
    pub const fn from_micros(micros: u64) -> Duration {
        Duration { micros }
    }

    /// Create a new `Duration` from a number of milliseconds.
    pub const fn from_millis(millis: u64) -> Duration {
        Duration { micros: millis * 1000 }
    }

    /// Create a new `Duration` from a number of seconds.
    pub const fn from_secs(secs: u64) -> Duration {
        Duration { micros: secs * 1000000 }
    }

    /// Create a new `Duration` from a number of seconds, as an `f64`.
    ///
    /// The fraction below one microsecond is truncated.
    ///
    /// # Panics
    /// Panics if `secs` is negative, NaN, or too large to represent.
    pub fn from_secs_f64(secs: f64) -> Duration {
        let micros = secs * 1000000.0;
        if !(micros >= 0.0 && micros <= u64::MAX as f64) {
            panic!("can not convert float seconds to Duration: value is either too big or NaN");
        }
        Duration { micros: micros as u64 }
    }

    /// Create a new `Duration` from a number of seconds, as an `f32`.
    ///
    /// The fraction below one microsecond is truncated.
    ///
    /// # Panics
    /// Panics if `secs` is negative, NaN, or too large to represent.
    pub fn from_secs_f32(secs: f32) -> Duration {
        Self::from_secs_f64(secs as f64)
    }

    /// Whether this is [`Duration::ZERO`].
    pub const fn is_zero(&self) -> bool {
        self.micros == 0
    }

    /// The number of whole seconds in this `Duration`.
    pub const fn as_secs(&self) -> u64 {
        self.micros / 1000000
    }

    /// The number of whole milliseconds in this `Duration`.
    pub const fn as_millis(&self) -> u64 {
        self.micros / 1000
    }

    /// The number of microseconds in this `Duration`.
    pub const fn as_micros(&self) -> u64 {
        self.micros
    }

    /// The number of nanoseconds in this `Duration`.
    pub const fn as_nanos(&self) -> u128 {
        self.micros as u128 * 1000
    }

    /// The number of milliseconds past [`as_secs`](Self::as_secs).
    ///
    /// Always less than 1000.
    pub const fn subsec_millis(&self) -> u32 {
        (self.micros / 1000 % 1000) as u32
    }

    /// The number of microseconds past [`as_secs`](Self::as_secs).
    ///
    /// Always less than 1000000.
    pub const fn subsec_micros(&self) -> u32 {
        (self.micros % 1000000) as u32
    }

    /// The number of nanoseconds past [`as_secs`](Self::as_secs).
    ///
    /// Always less than 1000000000, and always a whole number of microseconds.
    pub const fn subsec_nanos(&self) -> u32 {
        self.subsec_micros() * 1000
    }

    /// This `Duration` as a number of seconds, as an `f64`.
    pub fn as_secs_f64(&self) -> f64 {
        self.micros as f64 / 1000000.0
    }

    /// This `Duration` as a number of seconds, as an `f32`.
    pub fn as_secs_f32(&self) -> f32 {
        self.micros as f32 / 1000000.0
    }

    /// `self + rhs`, or `None` on overflow.
    pub const fn checked_add(&self, rhs: Duration) -> Option<Duration> {
        match self.micros.checked_add(rhs.micros) {
            Some(micros) => Some(Duration { micros }),
            None => None,
        }
    }

    /// `self - rhs`, or `None` if `rhs` is longer than `self`.
    pub const fn checked_sub(&self, rhs: Duration) -> Option<Duration> {
        match self.micros.checked_sub(rhs.micros) {
            Some(micros) => Some(Duration { micros }),
            None => None,
        }
    }

    /// `self * rhs`, or `None` on overflow.
    pub const fn checked_mul(&self, rhs: u32) -> Option<Duration> {
        match self.micros.checked_mul(rhs as u64) {
            Some(micros) => Some(Duration { micros }),
            None => None,
        }
    }

    /// `self / rhs`, or `None` if `rhs` is zero.
    pub const fn checked_div(&self, rhs: u32) -> Option<Duration> {
        match self.micros.checked_div(rhs as u64) {
            Some(micros) => Some(Duration { micros }),
            None => None,
        }
    }

    /// `self + rhs`, saturating at [`Duration::MAX`].
    pub const fn saturating_add(&self, rhs: Duration) -> Duration {
        Duration {
            micros: self.micros.saturating_add(rhs.micros),
        }
    }

    /// `self - rhs`, saturating at [`Duration::ZERO`].
    pub const fn saturating_sub(&self, rhs: Duration) -> Duration {
        Duration {
            micros: self.micros.saturating_sub(rhs.micros),
        }
    }

    /// `self * rhs`, saturating at [`Duration::MAX`].
    pub const fn saturating_mul(&self, rhs: u32) -> Duration {
        Duration {
            micros: self.micros.saturating_mul(rhs as u64),
        }
    }

    /// The absolute difference between `self` and `other`.
    pub const fn abs_diff(&self, other: Duration) -> Duration {
        Duration {
            micros: self.micros.abs_diff(other.micros),
        }
    }

    /// `self` multiplied by `rhs`.
    ///
    /// # Panics
    /// Panics if the result is negative, NaN, or too large to represent.
    pub fn mul_f64(&self, rhs: f64) -> Duration {
        Self::from_secs_f64(rhs * self.as_secs_f64())
    }

    /// `self` multiplied by `rhs`.
    ///
    /// # Panics
    /// Panics if the result is negative, NaN, or too large to represent.
    pub fn mul_f32(&self, rhs: f32) -> Duration {
        Self::from_secs_f64(rhs as f64 * self.as_secs_f64())
    }

    /// `self` divided by `rhs`.
    ///
    /// # Panics
    /// Panics if the result is negative, NaN, or too large to represent.
    pub fn div_f64(&self, rhs: f64) -> Duration {
        Self::from_secs_f64(self.as_secs_f64() / rhs)
    }

    /// `self` divided by `rhs`.
    ///
    /// # Panics
    /// Panics if the result is negative, NaN, or too large to represent.
    pub fn div_f32(&self, rhs: f32) -> Duration {
        Self::from_secs_f64(self.as_secs_f64() / rhs as f64)
    }

    /// The ratio of `self` to `rhs`.
    pub fn div_duration_f64(&self, rhs: Duration) -> f64 {
        self.as_secs_f64() / rhs.as_secs_f64()
    }

    /// The ratio of `self` to `rhs`.
    pub fn div_duration_f32(&self, rhs: Duration) -> f32 {
        self.as_secs_f32() / rhs.as_secs_f32()
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:03}s", self.as_secs(), self.subsec_millis())
    }
}

impl ops::Add<Duration> for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Duration {
        Duration::from_micros(self.micros + rhs.as_micros())
    }
}

impl ops::AddAssign<Duration> for Duration {
    fn add_assign(&mut self, rhs: Duration) {
        self.micros += rhs.as_micros();
    }
}

impl ops::Sub<Duration> for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Duration {
        Duration::from_micros(
            self.micros
                .checked_sub(rhs.as_micros())
                .expect("overflow when subtracting durations"),
        )
    }
}

impl ops::SubAssign<Duration> for Duration {
    fn sub_assign(&mut self, rhs: Duration) {
        self.micros = self
            .micros
            .checked_sub(rhs.as_micros())
            .expect("overflow when subtracting durations");
    }
}

impl ops::Mul<u32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: u32) -> Duration {
        Duration::from_micros(self.micros * rhs as u64)
    }
}

impl ops::Mul<Duration> for u32 {
    type Output = Duration;

    fn mul(self, rhs: Duration) -> Duration {
        rhs * self
    }
}

impl ops::MulAssign<u32> for Duration {
    fn mul_assign(&mut self, rhs: u32) {
        self.micros *= rhs as u64;
    }
}

impl ops::Div<u32> for Duration {
    type Output = Duration;

    fn div(self, rhs: u32) -> Duration {
        Duration::from_micros(self.micros / rhs as u64)
    }
}

impl ops::DivAssign<u32> for Duration {
    fn div_assign(&mut self, rhs: u32) {
        self.micros /= rhs as u64;
    }
}

impl From<::core::time::Duration> for Duration {
    fn from(other: ::core::time::Duration) -> Duration {
        Duration::from_micros(other.as_secs() * 1000000 + other.subsec_micros() as u64)
    }
}

impl From<Duration> for ::core::time::Duration {
    fn from(val: Duration) -> Self {
        ::core::time::Duration::from_micros(val.as_micros())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_instant_ops() {
        // std::ops::Add
        assert_eq!(
            Instant::from_millis(4) + Duration::from_millis(6),
            Instant::from_millis(10)
        );
        // std::ops::Sub
        assert_eq!(
            Instant::from_millis(7) - Duration::from_millis(5),
            Instant::from_millis(2)
        );
    }

    #[test]
    fn test_instant_getters() {
        let instant = Instant::from_millis(5674);
        assert_eq!(instant.as_secs(), 5);
        assert_eq!(instant.as_millis(), 5674);
        assert_eq!(instant.as_micros(), 5674000);
        assert_eq!(instant.subsec_millis(), 674);
        assert_eq!(instant.subsec_micros(), 674000);
    }

    #[test]
    fn test_instant_duration_since() {
        let a = Instant::from_millis(100);
        let b = Instant::from_millis(250);
        assert_eq!(b.duration_since(a), Duration::from_millis(150));
        assert_eq!(b - a, Duration::from_millis(150));
        // Saturates instead of returning the absolute difference.
        assert_eq!(a.duration_since(b), Duration::ZERO);
        assert_eq!(a - b, Duration::ZERO);
        assert_eq!(a.saturating_duration_since(b), Duration::ZERO);
        assert_eq!(a.checked_duration_since(b), None);
        assert_eq!(b.checked_duration_since(a), Some(Duration::from_millis(150)));
    }

    #[test]
    fn test_instant_checked() {
        let a = Instant::from_millis(100);
        assert_eq!(
            a.checked_add(Duration::from_millis(50)),
            Some(Instant::from_millis(150))
        );
        assert_eq!(a.checked_sub(Duration::from_millis(50)), Some(Instant::from_millis(50)));
        assert_eq!(Instant::MAX.checked_add(Duration::from_micros(1)), None);
        assert_eq!(Instant::MIN.checked_sub(Duration::from_micros(1)), None);
        // A duration that does not fit in an i64 at all.
        assert_eq!(a.checked_add(Duration::MAX), None);
        assert_eq!(a.checked_sub(Duration::MAX), None);
    }

    #[test]
    fn test_instant_display() {
        assert_eq!(format!("{}", Instant::from_millis(74)), "0.074s");
        assert_eq!(format!("{}", Instant::from_millis(5674)), "5.674s");
        assert_eq!(format!("{}", Instant::from_millis(5000)), "5.000s");
    }

    #[test]
    fn test_duration_ops() {
        // std::ops::Add
        assert_eq!(
            Duration::from_millis(40) + Duration::from_millis(2),
            Duration::from_millis(42)
        );
        // std::ops::Sub
        assert_eq!(
            Duration::from_millis(555) - Duration::from_millis(42),
            Duration::from_millis(513)
        );
        // std::ops::Mul
        assert_eq!(Duration::from_millis(13) * 22, Duration::from_millis(286));
        assert_eq!(22 * Duration::from_millis(13), Duration::from_millis(286));
        // std::ops::Div
        assert_eq!(Duration::from_millis(53) / 4, Duration::from_micros(13250));
    }

    #[test]
    fn test_duration_assign_ops() {
        let mut duration = Duration::from_millis(4735);
        duration += Duration::from_millis(1733);
        assert_eq!(duration, Duration::from_millis(6468));
        duration -= Duration::from_millis(1234);
        assert_eq!(duration, Duration::from_millis(5234));
        duration *= 4;
        assert_eq!(duration, Duration::from_millis(20936));
        duration /= 5;
        assert_eq!(duration, Duration::from_micros(4187200));
    }

    #[test]
    #[should_panic(expected = "overflow when subtracting durations")]
    fn test_sub_from_zero_overflow() {
        let _ = Duration::from_millis(0) - Duration::from_millis(1);
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn test_div_by_zero() {
        let _ = Duration::from_millis(4) / 0;
    }

    #[test]
    fn test_duration_getters() {
        let duration = Duration::from_millis(4934);
        assert_eq!(duration.as_secs(), 4);
        assert_eq!(duration.as_millis(), 4934);
        assert_eq!(duration.as_micros(), 4934000);
        assert_eq!(duration.as_nanos(), 4934000000);
        assert_eq!(duration.subsec_millis(), 934);
        assert_eq!(duration.subsec_micros(), 934000);
        assert_eq!(duration.subsec_nanos(), 934000000);
        assert!(!duration.is_zero());
        assert!(Duration::ZERO.is_zero());
    }

    #[test]
    fn test_duration_floats() {
        let duration = Duration::from_millis(4934);
        assert_eq!(duration.as_secs_f64(), 4.934);
        assert_eq!(duration.as_secs_f32(), 4.934);
        assert_eq!(Duration::from_secs_f64(4.934), duration);
        assert_eq!(Duration::from_secs_f32(0.5), Duration::from_millis(500));
        assert_eq!(Duration::from_secs(1).mul_f64(2.5), Duration::from_millis(2500));
        assert_eq!(Duration::from_secs(1).div_f64(4.0), Duration::from_millis(250));
        assert_eq!(Duration::from_secs(1).mul_f32(2.5), Duration::from_millis(2500));
        assert_eq!(Duration::from_secs(1).div_f32(4.0), Duration::from_millis(250));
        assert_eq!(Duration::from_secs(3).div_duration_f64(Duration::from_secs(2)), 1.5);
        assert_eq!(Duration::from_secs(3).div_duration_f32(Duration::from_secs(2)), 1.5);
    }

    #[test]
    #[should_panic(expected = "can not convert float seconds to Duration")]
    fn test_duration_from_secs_f64_negative() {
        let _ = Duration::from_secs_f64(-1.0);
    }

    #[test]
    fn test_duration_checked() {
        let a = Duration::from_millis(100);
        let b = Duration::from_millis(40);
        assert_eq!(a.checked_add(b), Some(Duration::from_millis(140)));
        assert_eq!(a.checked_sub(b), Some(Duration::from_millis(60)));
        assert_eq!(b.checked_sub(a), None);
        assert_eq!(Duration::MAX.checked_add(Duration::from_micros(1)), None);
        assert_eq!(a.checked_mul(3), Some(Duration::from_millis(300)));
        assert_eq!(Duration::MAX.checked_mul(2), None);
        assert_eq!(a.checked_div(4), Some(Duration::from_millis(25)));
        assert_eq!(a.checked_div(0), None);
    }

    #[test]
    fn test_duration_saturating() {
        let a = Duration::from_millis(100);
        let b = Duration::from_millis(40);
        assert_eq!(a.saturating_add(b), Duration::from_millis(140));
        assert_eq!(Duration::MAX.saturating_add(a), Duration::MAX);
        assert_eq!(a.saturating_sub(b), Duration::from_millis(60));
        assert_eq!(b.saturating_sub(a), Duration::ZERO);
        assert_eq!(a.saturating_mul(2), Duration::from_millis(200));
        assert_eq!(Duration::MAX.saturating_mul(2), Duration::MAX);
        assert_eq!(a.abs_diff(b), Duration::from_millis(60));
        assert_eq!(b.abs_diff(a), Duration::from_millis(60));
    }

    #[test]
    fn test_duration_conversions() {
        let mut std_duration = ::core::time::Duration::from_millis(4934);
        let duration: Duration = std_duration.into();
        assert_eq!(duration, Duration::from_millis(4934));
        assert_eq!(Duration::from(std_duration), Duration::from_millis(4934));

        std_duration = duration.into();
        assert_eq!(std_duration, ::core::time::Duration::from_millis(4934));
    }
}

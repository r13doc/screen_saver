use std::ops::Range;
use std::time::Duration;
use rand::Rng;
use rand::rng;


#[derive(Debug)]
pub enum Ticks {
    Minutes,
    Hours, // 1.5 one hour and half
}
pub enum SetInterval {
    Ticks(Ticks),
    Range(Range<Ticks>),
    On_Request,
}

fn minutes(time: u32) -> Duration {
    let sec: u64 = (time * 60).into();
    Duration::from_secs(sec)
}
fn hours(time: f32) -> Duration {
    // aka example 1.5 -> one hour and 30 min
    let minutes: u64 = (time * 60_f32) as u64;
    let sec = minutes * 60;
    Duration::from_secs(sec)
}

impl SetInterval {
    // ticks in hours -> 1.5 one hour and 30 minutes, float numbers
    pub fn ticks_hours(&self, num: f32) -> Option<Duration> {
        if let SetInterval::Ticks(Ticks::Hours) = self {
            return Some(hours(num))
        }
        None
    }
    pub fn ticks_minutes(&self, num: u32) -> Option<Duration> {
        if let SetInterval::Ticks(Ticks::Minutes) = self {
            return Some(minutes(num))
        }
        None
    }
    pub fn range_minutes(&self, range: Range<usize> ) -> Option<Duration> {

        if let SetInterval::Range(Range {start:Ticks::Minutes,
                                         end: Ticks::Minutes}) = self {
            let rng = rand::rng()
                .random_range(range.start..range.end);
            let rng_min = minutes(rng as u32);
            return Some(rng_min);
        }
        None
    }
    pub fn range_hours(&self, range: Range<f32> ) -> Option<Duration> {
        if let SetInterval::Range(Range {start:Ticks::Hours,
                                      end: Ticks::Hours}) = self {
            let rng = rand::rng()
                .random_range(range.start..range.end);
            println!("{:?}", rng);
            let rng_hour = hours(rng);
            return Some(rng_hour);
        }
        None
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min() {
        let tes = minutes(100);
        assert_eq!(tes.as_secs(), 6000);
    }
    #[test]
    fn hour() {
        let tes = hours(1.25);
        assert_eq!(tes.as_secs(), 75 * 60);
    }
    #[test]
    fn tick_hour() {
        let interval = SetInterval::Ticks(Ticks::Hours);
        // 0.3 half
        let tick = interval.ticks_hours(1.5);
        assert_eq!(tick.unwrap().as_secs(), (1.5 * 60.0 * 60.0) as u64);
    }
    #[test]
    fn tick_min() {
        let interval = SetInterval::Ticks(Ticks::Minutes);
        let tick = interval.ticks_minutes(12);
        assert_eq!(tick.unwrap().as_secs(), 12 * 60);
    }
    #[test]
    fn range_min() {
        let range_min = SetInterval::Range(Ticks::Minutes..Ticks::Minutes);
        let res = range_min.range_minutes(35..65);
        let res_2 = range_min.range_minutes(35..65);
        assert_ne!(res, res_2);
    }
    #[test]
    fn range_hour() {
        let range_hour = SetInterval::Range(Ticks::Hours..Ticks::Hours);
        let res = range_hour.range_hours(3.5..4.5);
        let res_2 = range_hour.range_hours(3.5..4.5);
        assert_ne!(res, res_2)
    }
}

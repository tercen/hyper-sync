use std::fmt::{self, Display};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use time;

/// A timestamp with HTTP formatting and parsing
//   Prior to 1995, there were three different formats commonly used by
//   servers to communicate timestamps.  For compatibility with old
//   implementations, all three are defined here.  The preferred format is
//   a fixed-length and single-zone subset of the date and time
//   specification used by the Internet Message Format [RFC5322].
//
//     HTTP-date    = IMF-fixdate / obs-date
//
//   An example of the preferred format is
//
//     Sun, 06 Nov 1994 08:49:37 GMT    ; IMF-fixdate
//
//   Examples of the two obsolete formats are
//
//     Sunday, 06-Nov-94 08:49:37 GMT   ; obsolete RFC 850 format
//     Sun Nov  6 08:49:37 1994         ; ANSI C's asctime() format
//
//   A recipient that parses a timestamp value in an HTTP header field
//   MUST accept all three HTTP-date formats.  When a sender generates a
//   header field that contains one or more timestamps defined as
//   HTTP-date, the sender MUST generate those timestamps in the
//   IMF-fixdate format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HttpDate(time::Tm);

impl FromStr for HttpDate {
    type Err = ::Error;
    fn from_str(s: &str) -> ::Result<HttpDate> {
        match time::strptime(s, "%a, %d %b %Y %T %Z").or_else(|_| {
            time::strptime(s, "%A, %d-%b-%y %T %Z")
            }).or_else(|_| {
                time::strptime(s, "%c")
                }) {
                    Ok(t) => Ok(HttpDate(t)),
                    Err(_) => Err(::Error::Header),
                    }
    }
}

impl Display for HttpDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0.to_utc().rfc822(), f)
    }
}

impl From<SystemTime> for HttpDate {
    fn from(sys: SystemTime) -> HttpDate {
        let tmspec = match sys.duration_since(UNIX_EPOCH) {
            Ok(dur) => {
                time::Timespec::new(dur.as_secs() as i64, dur.subsec_nanos() as i32)
            },
            Err(err) => {
                let neg = err.duration();
                time::Timespec::new(-(neg.as_secs() as i64), -(neg.subsec_nanos() as i32))
            },
        };
        HttpDate(time::at_utc(tmspec))
    }
}

impl From<HttpDate> for SystemTime {
    fn from(date: HttpDate) -> SystemTime {
        let spec = date.0.to_timespec();
        if spec.sec >= 0 {
            UNIX_EPOCH + Duration::new(spec.sec as u64, spec.nsec as u32)
        } else {
            UNIX_EPOCH - Duration::new(spec.sec as u64, spec.nsec as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use time::Tm;
    use super::HttpDate;

    const NOV_07: HttpDate = HttpDate(Tm {
        tm_nsec: 0,
        tm_sec: 37,
        tm_min: 48,
        tm_hour: 8,
        tm_mday: 7,
        tm_mon: 10,
        tm_year: 94,
        tm_wday: 0,
        tm_isdst: 0,
        tm_yday: 0,
        tm_utcoff: 0,
    });

    #[test]
    fn test_imf_fixdate() {
        assert_eq!("Sun, 07 Nov 1994 08:48:37 GMT".parse::<HttpDate>().unwrap(), NOV_07);
    }

    #[test]
    fn test_rfc_850() {
        assert_eq!("Sunday, 07-Nov-94 08:48:37 GMT".parse::<HttpDate>().unwrap(), NOV_07);
    }

    #[test]
    fn test_asctime() {
        assert_eq!("Sun Nov  7 08:48:37 1994".parse::<HttpDate>().unwrap(), NOV_07);
    }

    #[test]
    fn test_no_date() {
        assert!("this-is-no-date".parse::<HttpDate>().is_err());
    }
}

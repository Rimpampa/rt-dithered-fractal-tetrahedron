use super::*;
use std::ops::*;

impl Sub for ColorDiff {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
        }
    }
}

impl Mul<i16> for ColorDiff {
    type Output = ColorDiff;

    fn mul(self, rhs: i16) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl Div<i16> for ColorDiff {
    type Output = ColorDiff;

    fn div(self, rhs: i16) -> Self::Output {
        Self {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
    }
}

impl Sub for Color<'_> {
    type Output = ColorDiff;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            r: *self.r() as i16 - *rhs.r() as i16,
            g: *self.g() as i16 - *rhs.g() as i16,
            b: *self.b() as i16 - *rhs.b() as i16,
        }
    }
}

impl Add<ColorDiff> for Color<'_> {
    type Output = Self;

    fn add(self, rhs: ColorDiff) -> Self::Output {
        Self::Output {
            rgb: [
                if rhs.r.is_negative() {
                    self.r().saturating_sub(rhs.r.abs() as u8)
                } else {
                    self.r().saturating_add(rhs.r.abs() as u8)
                },
                if rhs.g.is_negative() {
                    self.g().saturating_sub(rhs.g.abs() as u8)
                } else {
                    self.g().saturating_add(rhs.g.abs() as u8)
                },
                if rhs.b.is_negative() {
                    self.b().saturating_sub(rhs.b.abs() as u8)
                } else {
                    self.b().saturating_add(rhs.b.abs() as u8)
                },
            ]
            .into(),
        }
    }
}

impl AddAssign<ColorDiff> for Color<'_> {
    fn add_assign(&mut self, rhs: ColorDiff) {
        self.set(self.clone() + rhs);
    }
}

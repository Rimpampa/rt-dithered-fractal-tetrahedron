#![allow(unused)]

macro_rules! rgb {
    (#$n:tt) => {
        Color::from_str(stringify!($n)).unwrap()
    };
    ($r:expr, $g:expr, $b: expr) => {
        Color {
            rgb: [$r, $g, $b].into(),
        }
    };
}

mod ops;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum Mutable<'a, T> {
    Borrowed(&'a mut T),
    Owned(T),
}

impl<'a, 'b, T: PartialEq> PartialEq<Mutable<'a, T>> for Mutable<'b, T> {
    fn eq(&self, other: &Mutable<T>) -> bool {
        self.get() == other.get()
    }
}

impl<'a, T> Mutable<'a, T> {
    pub fn get_mut(&mut self) -> &mut T {
        match self {
            Self::Borrowed(v) => v,
            Self::Owned(ref mut v) => v,
        }
    }

    pub fn get(&self) -> &T {
        match self {
            Self::Borrowed(v) => v,
            Self::Owned(ref v) => v,
        }
    }

    pub fn set(&mut self, value: T) {
        *self.get_mut() = value;
    }
}

impl<'a, T: Copy> Mutable<'a, T> {
    pub fn copy<'b>(&self) -> Mutable<'b, T> {
        (*self.get()).into()
    }

    pub fn copy_from<'b>(&mut self, value: &Mutable<'b, T>) {
        self.set(*value.get())
    }
}

impl<T: Clone> Mutable<'_, T> {
    fn clone(&self) -> Mutable<'static, T> {
        self.get().clone().into()
    }

    fn clone_from<'b>(&mut self, value: &Mutable<'b, T>) {
        self.set(value.get().clone())
    }
}

impl<T> From<T> for Mutable<'_, T> {
    fn from(from: T) -> Self {
        Self::Owned(from)
    }
}

impl<'a, T> From<&'a mut T> for Mutable<'a, T> {
    fn from(from: &'a mut T) -> Self {
        Self::Borrowed(from)
    }
}

#[derive(Debug, PartialEq)]
pub struct Color<'a> {
    pub rgb: Mutable<'a, [u8; 3]>,
}

impl<'a> From<&'a mut [u8; 3]> for Color<'a> {
    fn from(rgb: &'a mut [u8; 3]) -> Self {
        Self { rgb: rgb.into() }
    }
}

impl From<[u8; 3]> for Color<'static> {
    fn from(rgb: [u8; 3]) -> Self {
        Self { rgb: rgb.into() }
    }
}

fn hex_value(ch: &u8) -> Option<u8> {
    match ch {
        ch if ch.is_ascii_digit() => Some(ch - b'0'),
        ch if ch.is_ascii_alphabetic() => Some((ch.to_ascii_lowercase() - b'a') + 10),
        _ => None,
    }
}

impl FromStr for Color<'static> {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if !bytes.iter().all(u8::is_ascii_hexdigit) {
            Err("The value contains a non hexadecimal digit")
        } else if bytes.len() != 6 {
            Err("The value must be made of 6 hex digits (RRGGBB)")
        } else {
            let mut hex = bytes.iter().map(hex_value).map(Option::unwrap);
            Ok(Self {
                rgb: [
                    (hex.next().unwrap() << 4) + hex.next().unwrap(),
                    (hex.next().unwrap() << 4) + hex.next().unwrap(),
                    (hex.next().unwrap() << 4) + hex.next().unwrap(),
                ]
                .into(),
            })
        }
    }
}

impl<'a> Color<'a> {
    pub fn set<'b>(&mut self, other: Color<'b>) {
        self.rgb.copy_from(&other.rgb);
    }

    pub fn r(&self) -> &u8 {
        &self.rgb.get()[0]
    }

    pub fn g(&self) -> &u8 {
        &self.rgb.get()[1]
    }

    pub fn b(&self) -> &u8 {
        &self.rgb.get()[2]
    }

    pub fn clone(&self) -> Color<'static> {
        Color {
            rgb: self.rgb.clone(),
        }
    }
}

impl fmt::Display for Color<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "#{:2x}{:2x}{:2x}", self.r(), self.g(), self.b())
    }
}

#[derive(Clone, Copy)]
pub struct ColorDiff {
    pub r: i16,
    pub g: i16,
    pub b: i16,
}

impl ColorDiff {
    pub fn length(&self) -> u32 {
        let rsq = self.r.abs() as u32 * self.r.abs() as u32;
        let gsq = self.g.abs() as u32 * self.g.abs() as u32;
        let bsq = self.b.abs() as u32 * self.b.abs() as u32;
        rsq + gsq + bsq
    }
}

pub struct Palette {
    colors: Vec<Color<'static>>,
}

impl Palette {
    pub fn new<T: Into<Vec<Color<'static>>>>(colors: T) -> Self {
        Self {
            colors: colors.into(),
        }
    }

    pub fn closest(&self, color: &Color) -> usize {
        self.colors
            .iter()
            .enumerate()
            .map(|(i, c)| (i, (c.clone() - color.clone()).length()))
            .min_by_key(|&(_, c)| c)
            .unwrap()
            .0
    }

    pub fn colors(&self) -> &[Color] {
        &self.colors
    }

    pub fn find(&self, color: &Color) -> Option<usize> {
        self.colors.iter().position(|c| c == color)
    }
}

#[test]
fn test() {
    println!("{:?}", rgb!(#0000ff));
    println!("{:?}", rgb!(10, 20, 30));
}

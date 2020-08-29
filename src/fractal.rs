use crate::math;
use std::ops::*;

/// A simple `Point` composed of three coordinates (3-dimensional) `x`, `y` and `z`
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point {
    /// Constructs a `Point` from the three values of `x`, `y` and `z`
    pub fn new(x: f32, y: f32, z: f32) -> Point {
        Point { x, y, z }
    }
}

/// A polygon composed of three edges and three points `a`, `b` and `c`
/*
*         /\
*        /  \
*       /    \
*      /______\
*/
#[repr(C, packed)]
#[derive(Debug)]
pub struct Triangle {
    pub a: Point,
    pub b: Point,
    pub c: Point,
}

impl Triangle {
    /// Constructs a new `Triangle` from the given `Point`s
    pub fn new(a: Point, b: Point, c: Point) -> Triangle {
        Triangle { a, b, c }
    }

    /// Splits the triangle into three other leaving a gap at the center
    pub fn sierpinski_split(self) -> [Triangle; 3] {
        // Find the points at the center of each edge
        let d = (self.a + self.b) * 0.5;
        let e = (self.b + self.c) * 0.5;
        let f = (self.c + self.a) * 0.5;
        [
            Triangle::new(self.a, d, f),
            Triangle::new(self.b, d, e),
            Triangle::new(self.c, e, f),
        ]
    }
}
/// A polyhedron composed of of four triangluar faces, six edges and four points (`a`, `b`, `c`, and `d`)
/*
*            .|\
*          .' | \
*        .'   |  \
*      .:_____|___\
*       `-._  |  /
*           `-|/
*/
#[repr(C, packed)]
#[derive(Debug)]
pub struct Tetrahedron {
    // a: Point,
    // b: Point,
    // c: Point,
    // d: Point,
    a: Triangle,
    b: Triangle,
    c: Triangle,
    d: Triangle,
}

impl Tetrahedron {
    /// Constructs a new `Tetrahedron` from the given `Points`s `a`, `b`, `c` and `d` where `d` is the apex
    pub fn new(a: Point, b: Point, c: Point, d: Point) -> Tetrahedron {
        let (a, b, c, d) = (
            Triangle::new(a, b, c),
            Triangle::new(a, b, d),
            Triangle::new(b, c, d),
            Triangle::new(c, a, d),
        );
        Tetrahedron { a, b, c, d }
    }

    /// Constructs a regular tetrahedron (a `Tetrahedron` made of `Triangle`s which have the same side length)
    pub fn regular(base: Point, height: f32, angle: f32) -> Tetrahedron {
        use std::f32::consts::*;

        // -------------------------------------------------
        // height = sqrt(2/3) * side
        //        thus
        // side = sqrt(3/2) * height

        // len = sqrt(side^2 - height^2) =
        //     = sqrt(sqrt(3/2)^2 * height^2 - height^2) =
        //     = sqrt(3/2 * height^2 - height^2) =
        //     = sqrt((3/2 - 1) * height^2) =
        //     = sqrt((1/2) * height^2) =
        //     = height * sqrt(1/2)
        // -------------------------------------------------

        // Distance from the center of the triangle and one of its points
        let len = height * FRAC_1_SQRT_2;

        // The base is constructed using the sin and cosine goniometric functions
        // When watching from the center of the base each of its points is spaced
        // by 120 degrees from each other, thus we can compute
        let a = Point::new(
            len.mul_add(angle.cos(), base.x),
            base.y,
            len.mul_add(angle.sin(), base.z),
        );
        let alpha = angle + math::TWO_THIRDS_PI;
        let b = Point::new(
            len.mul_add(alpha.cos(), base.x),
            base.y,
            len.mul_add(alpha.sin(), base.z),
        );
        let alpha = angle - math::TWO_THIRDS_PI;
        let c = Point::new(
            len.mul_add(alpha.cos(), base.x),
            base.y,
            len.mul_add(alpha.sin(), base.z),
        );
        // The apex is computed by going upwards from the origin by the distance of `height`
        let d = Point::new(base.x, base.y + height, base.z);

        Tetrahedron::new(a, b, c, d)
    }

    /// Splits the `Tetrahedron` into four other leaving a gap at the center
    #[allow(clippy::many_single_char_names)]
    pub fn sierpinski_split(self) -> [Tetrahedron; 4] {
        // Find the points at the center of each edge
        let (a, b, c, d) = (self.a.a, self.a.b, self.a.c, self.b.c);
        let e = (a + b) * 0.5;
        let f = (b + c) * 0.5;
        let g = (c + a) * 0.5;
        let h = (a + d) * 0.5;
        let i = (b + d) * 0.5;
        let j = (c + d) * 0.5;
        [
            Tetrahedron::new(a, e, g, h),
            Tetrahedron::new(b, f, e, i),
            Tetrahedron::new(c, g, f, j),
            Tetrahedron::new(d, h, i, j),
        ]
    }
}

impl Add for Point {
    type Output = Point;
    fn add(mut self, other: Point) -> Point {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
        self
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(mut self, other: Point) -> Point {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
        self
    }
}

impl Add<f32> for Point {
    type Output = Point;
    fn add(mut self, other: f32) -> Point {
        self.x += other;
        self.y += other;
        self.z += other;
        self
    }
}

impl Sub<f32> for Point {
    type Output = Point;
    fn sub(mut self, other: f32) -> Point {
        self.x -= other;
        self.y -= other;
        self.z -= other;
        self
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(mut self, other: f32) -> Point {
        self.x *= other;
        self.y *= other;
        self.z *= other;
        self
    }
}

impl Div<f32> for Point {
    type Output = Point;
    fn div(mut self, other: f32) -> Point {
        self.x /= other;
        self.y /= other;
        self.z /= other;
        self
    }
}

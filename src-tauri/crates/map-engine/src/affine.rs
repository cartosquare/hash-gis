//Copyright (c) 2014, Sean C. Gillies
//All rights reserved.
//
//Redistribution and use in source and binary forms, with or without
//modification, are permitted provided that the following conditions are met:
//
//    * Redistributions of source code must retain the above copyright
//      notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above copyright
//      notice, this list of conditions and the following disclaimer in the
//      documentation and/or other materials provided with the distribution.
//    * Neither the name of Sean C. Gillies nor the names of
//      its contributors may be used to endorse or promote products derived from
//      this software without specific prior written permission.
//
//THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
//AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
//IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
//ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
//LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
//CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
//SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
//INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
//CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
//ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
//POSSIBILITY OF SUCH DAMAGE.
/*!Matrices describing affine transformation of the plane.

Many of the functions are extracted from <https://github.com/rasterio/affine>.
*/
use crate::errors::MapEngineError;
use num_traits::AsPrimitive;
use serde::{Deserialize, Serialize};
use std::ops::Mul;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct GeoTransform {
    pub geo: [f64; 6],
}

impl GeoTransform {
    pub fn new(geo: &[f64; 6]) -> Self {
        Self { geo: *geo }
    }
    #[allow(clippy::many_single_char_names)]
    pub fn from_gdal(geo: &[f64; 6]) -> Self {
        let c = geo[0];
        let a = geo[1];
        let b = geo[2];
        let f = geo[3];
        let d = geo[4];
        let e = geo[5];
        Self::new(&[a, b, c, d, e, f])
    }

    pub fn translation(tx: f64, ty: f64) -> Self {
        let geo = [1.0, 0.0, tx, 0.0, 1.0, ty];
        Self::new(&geo)
    }

    pub fn scale(sx: f64, sy: f64) -> Self {
        let geo = [sx, 0.0, 0.0, 0.0, sy, 0.0];
        Self::new(&geo)
    }

    pub fn shear(sx: f64, sy: f64) -> Self {
        let geo = [
            1.0,
            sx.to_radians().tan(),
            0.0,
            sy.to_radians().tan(),
            1.0,
            0.0,
        ];
        Self::new(&geo)
    }

    #[allow(clippy::many_single_char_names)]
    pub fn to_tuple(&self) -> (f64, f64, f64, f64, f64, f64) {
        let a = self.geo[0];
        let b = self.geo[1];
        let c = self.geo[2];
        let d = self.geo[3];
        let e = self.geo[4];
        let f = self.geo[5];
        (a, b, c, d, e, f)
    }

    pub fn to_array(&self) -> [f64; 6] {
        self.geo
    }

    pub fn inv(&self) -> Result<Self, MapEngineError> {
        let (sa, sb, sc, sd, se, sf) = self.to_tuple();

        let determinant = self.determinant();
        if determinant == 0.0 {
            return Err(MapEngineError::AffineError("Determinant is zero".into()));
        }

        let idet = 1.0 / determinant;

        let ra = se * idet;
        let rb = -sb * idet;
        let rd = -sd * idet;
        let re = sa * idet;

        let geo = [ra, rb, -sc * ra - sf * rb, rd, re, -sc * rd - sf * re];
        Ok(Self::new(&geo))
    }

    pub fn determinant(&self) -> f64 {
        let (a, b, _, d, e, _) = self.to_tuple();
        a * e - b * d
    }

    pub fn xy(&self, row: u32, col: u32) -> (f64, f64) {
        let (coff, roff) = (0.5, 0.5);
        let trans = Self::translation(coff, roff);
        let tmp = self * &trans;
        &tmp * (col as f64, row as f64)
    }

    pub fn rowcol(&self, x: f64, y: f64) -> Result<(i32, i32), MapEngineError> {
        let eps = 2.220446049250313e-16;

        let inv = self.inv()?;
        let (fcol, frow) = &inv * (x + eps, y + eps);

        Ok((frow.floor() as i32, fcol.floor() as i32))
    }

    #[allow(clippy::many_single_char_names)]
    pub fn to_gdal(&self) -> Self {
        let (a, b, c, d, e, f) = self.to_tuple();
        let geo = [c, a, b, f, d, e];
        Self::new(&geo)
    }

    pub fn xoff(&self) -> f64 {
        self.geo[2]
    }
    pub fn yoff(&self) -> f64 {
        self.geo[5]
    }
}

impl Mul<&GeoTransform> for &GeoTransform {
    type Output = GeoTransform;

    fn mul(self, rhs: &GeoTransform) -> GeoTransform {
        let (sa, sb, sc, sd, se, sf) = self.to_tuple();
        let (oa, ob, oc, od, oe, of) = rhs.to_tuple();
        let geo = [
            sa * oa + sb * od,
            sa * ob + sb * oe,
            sa * oc + sb * of + sc,
            sd * oa + se * od,
            sd * ob + se * oe,
            sd * oc + se * of + sf,
        ];
        GeoTransform::new(&geo)
    }
}

impl<T> Mul<(T, T)> for &GeoTransform
where
    T: AsPrimitive<f64>,
{
    type Output = (f64, f64);

    fn mul(self, rhs: (T, T)) -> Self::Output {
        let (sa, sb, sc, sd, se, sf) = self.to_tuple();
        let (vx, vy) = rhs;
        let vx: f64 = vx.as_();
        let vy: f64 = vy.as_();
        (vx * sa + vy * sb + sc, vx * sd + vy * se + sf)
    }
}

impl From<gdal::GeoTransform> for GeoTransform {
    #[allow(clippy::many_single_char_names)]
    fn from(geo: gdal::GeoTransform) -> Self {
        let c = geo[0];
        let a = geo[1];
        let b = geo[2];
        let f = geo[3];
        let d = geo[4];
        let e = geo[5];
        Self::new(&[a, b, c, d, e, f])
    }
}

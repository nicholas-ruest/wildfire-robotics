//! Framework-independent geospatial primitives with explicit reference systems.

use crate::numeric::{Finite, NumericError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// An EPSG coordinate reference system identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct CoordinateReferenceSystem(u32);

impl CoordinateReferenceSystem {
    /// WGS 84 geographic longitude/latitude coordinates.
    pub const WGS84: Self = Self(4326);

    /// Creates a reference system from a non-zero EPSG registry code.
    pub fn epsg(code: u32) -> Result<Self, GeometryError> {
        if code == 0 {
            Err(GeometryError::InvalidCrs)
        } else {
            Ok(Self(code))
        }
    }

    #[must_use]
    /// Returns the EPSG registry code.
    pub const fn epsg_code(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for CoordinateReferenceSystem {
    type Error = GeometryError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::epsg(value)
    }
}

impl From<CoordinateReferenceSystem> for u32 {
    fn from(value: CoordinateReferenceSystem) -> Self {
        value.epsg_code()
    }
}

/// Datum used to interpret an altitude measurement.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AltitudeReference {
    /// Orthometric height relative to mean sea level.
    MeanSeaLevel,
    /// Ellipsoidal height relative to the WGS 84 ellipsoid.
    Wgs84Ellipsoid,
    /// Height relative to the local terrain surface.
    AboveGroundLevel,
}

/// A finite altitude expressed in metres and tied to an explicit datum.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Altitude {
    metres: Finite,
    reference: AltitudeReference,
}

impl Altitude {
    /// Creates a finite altitude with an explicit vertical reference.
    pub fn new(metres: f64, reference: AltitudeReference) -> Result<Self, GeometryError> {
        Ok(Self {
            metres: Finite::new(metres)?,
            reference,
        })
    }

    #[must_use]
    /// Returns the altitude in metres.
    pub const fn metres(self) -> f64 {
        self.metres.get()
    }

    #[must_use]
    /// Returns the altitude's vertical reference.
    pub const fn reference(self) -> AltitudeReference {
        self.reference
    }
}

/// A point whose axes are interpreted only in the declared CRS.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct GeoPoint {
    x: Finite,
    y: Finite,
    crs: CoordinateReferenceSystem,
    altitude: Option<Altitude>,
}

#[derive(Deserialize)]
struct GeoPointWire {
    x: Finite,
    y: Finite,
    crs: CoordinateReferenceSystem,
    altitude: Option<Altitude>,
}

impl<'de> Deserialize<'de> for GeoPoint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = GeoPointWire::deserialize(deserializer)?;
        Self::new(wire.x.get(), wire.y.get(), wire.crs, wire.altitude)
            .map_err(serde::de::Error::custom)
    }
}

impl GeoPoint {
    /// Creates a finite point, validating known axis bounds for WGS 84.
    pub fn new(
        x: f64,
        y: f64,
        crs: CoordinateReferenceSystem,
        altitude: Option<Altitude>,
    ) -> Result<Self, GeometryError> {
        let x = Finite::new(x)?;
        let y = Finite::new(y)?;
        if crs == CoordinateReferenceSystem::WGS84
            && (!(-180.0..=180.0).contains(&x.get()) || !(-90.0..=90.0).contains(&y.get()))
        {
            return Err(GeometryError::CoordinateOutOfRange);
        }
        Ok(Self {
            x,
            y,
            crs,
            altitude,
        })
    }

    #[must_use]
    /// Returns the first-axis coordinate (longitude for WGS 84).
    pub const fn x(self) -> f64 {
        self.x.get()
    }
    #[must_use]
    /// Returns the second-axis coordinate (latitude for WGS 84).
    pub const fn y(self) -> f64 {
        self.y.get()
    }
    #[must_use]
    /// Returns the coordinate reference system.
    pub const fn crs(self) -> CoordinateReferenceSystem {
        self.crs
    }
    #[must_use]
    /// Returns the optional explicitly referenced altitude.
    pub const fn altitude(self) -> Option<Altitude> {
        self.altitude
    }
}

/// A simple polygon represented by a canonical exterior ring.
///
/// The closing point is not stored; callers may supply it and it is removed.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GeoPolygon {
    exterior: Vec<GeoPoint>,
    crs: CoordinateReferenceSystem,
}

#[derive(Deserialize)]
struct GeoPolygonWire {
    exterior: Vec<GeoPoint>,
    crs: CoordinateReferenceSystem,
}

impl<'de> Deserialize<'de> for GeoPolygon {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = GeoPolygonWire::deserialize(deserializer)?;
        let polygon = Self::new(wire.exterior).map_err(serde::de::Error::custom)?;
        if polygon.crs != wire.crs {
            return Err(serde::de::Error::custom(GeometryError::MixedCrs));
        }
        Ok(polygon)
    }
}

impl GeoPolygon {
    /// Validates and canonicalizes a simple exterior ring.
    pub fn new(mut exterior: Vec<GeoPoint>) -> Result<Self, GeometryError> {
        if exterior.first() == exterior.last() && exterior.len() > 1 {
            exterior.pop();
        }
        if exterior.len() < 3 {
            return Err(GeometryError::TooFewVertices);
        }
        let crs = exterior[0].crs();
        if exterior.iter().any(|point| point.crs() != crs) {
            return Err(GeometryError::MixedCrs);
        }
        if exterior
            .iter()
            .enumerate()
            .any(|(index, point)| exterior.iter().skip(index + 1).any(|other| point == other))
        {
            return Err(GeometryError::DuplicateVertex);
        }
        if self_intersects(&exterior) {
            return Err(GeometryError::SelfIntersection);
        }
        if is_near_zero(signed_twice_area(&exterior)) {
            return Err(GeometryError::DegenerateRing);
        }
        Ok(Self { exterior, crs })
    }

    #[must_use]
    /// Returns the unclosed canonical exterior ring.
    pub fn exterior(&self) -> &[GeoPoint] {
        &self.exterior
    }

    #[must_use]
    /// Returns the coordinate reference system shared by all vertices.
    pub const fn crs(&self) -> CoordinateReferenceSystem {
        self.crs
    }
}

fn signed_twice_area(points: &[GeoPoint]) -> f64 {
    (0..points.len()).fold(0.0, |area, index| {
        let next = (index + 1) % points.len();
        area + points[index].x() * points[next].y() - points[next].x() * points[index].y()
    })
}

fn orientation(a: GeoPoint, b: GeoPoint, c: GeoPoint) -> f64 {
    (b.y() - a.y()) * (c.x() - b.x()) - (b.x() - a.x()) * (c.y() - b.y())
}

fn is_near_zero(value: f64) -> bool {
    value.abs() <= f64::EPSILON
}

fn on_segment(a: GeoPoint, b: GeoPoint, c: GeoPoint) -> bool {
    b.x() >= a.x().min(c.x())
        && b.x() <= a.x().max(c.x())
        && b.y() >= a.y().min(c.y())
        && b.y() <= a.y().max(c.y())
}

fn segments_intersect(a: GeoPoint, b: GeoPoint, c: GeoPoint, d: GeoPoint) -> bool {
    let o1 = orientation(a, b, c);
    let o2 = orientation(a, b, d);
    let o3 = orientation(c, d, a);
    let o4 = orientation(c, d, b);
    (o1.is_sign_positive() != o2.is_sign_positive()
        && o3.is_sign_positive() != o4.is_sign_positive())
        || (is_near_zero(o1) && on_segment(a, c, b))
        || (is_near_zero(o2) && on_segment(a, d, b))
        || (is_near_zero(o3) && on_segment(c, a, d))
        || (is_near_zero(o4) && on_segment(c, b, d))
}

fn self_intersects(points: &[GeoPoint]) -> bool {
    let count = points.len();
    for first in 0..count {
        let first_next = (first + 1) % count;
        for second in (first + 1)..count {
            let second_next = (second + 1) % count;
            if first == second || first_next == second || second_next == first {
                continue;
            }
            if segments_intersect(
                points[first],
                points[first_next],
                points[second],
                points[second_next],
            ) {
                return true;
            }
        }
    }
    false
}

/// Stable geometry boundary failures.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum GeometryError {
    /// EPSG code zero was supplied.
    #[error("EPSG code must be non-zero")]
    InvalidCrs,
    /// A coordinate exceeded the declared CRS's known bounds.
    #[error("coordinate is outside the declared CRS bounds")]
    CoordinateOutOfRange,
    /// A ring had fewer than three distinct vertices.
    #[error("polygon requires at least three distinct vertices")]
    TooFewVertices,
    /// Polygon vertices used different coordinate reference systems.
    #[error("all polygon vertices must use the same CRS")]
    MixedCrs,
    /// A ring contained the same vertex more than once.
    #[error("polygon contains a duplicate vertex")]
    DuplicateVertex,
    /// A ring's vertices enclosed no area.
    #[error("polygon ring has zero area")]
    DegenerateRing,
    /// Non-adjacent edges of a ring intersected.
    #[error("polygon ring intersects itself")]
    SelfIntersection,
    /// A coordinate or altitude was not finite.
    #[error(transparent)]
    InvalidNumber(#[from] NumericError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> Result<GeoPoint, GeometryError> {
        GeoPoint::new(x, y, CoordinateReferenceSystem::WGS84, None)
    }

    #[test]
    fn should_accept_wgs84_coordinate_boundaries() {
        for (longitude, latitude) in [(-180.0, -90.0), (180.0, 90.0)] {
            assert!(point(longitude, latitude).is_ok());
        }
    }

    #[test]
    fn should_reject_wgs84_coordinates_beyond_boundaries() {
        for (longitude, latitude) in [(-180.1, 0.0), (180.1, 0.0), (0.0, -90.1), (0.0, 90.1)] {
            assert_eq!(
                point(longitude, latitude),
                Err(GeometryError::CoordinateOutOfRange)
            );
        }
    }

    #[test]
    fn should_reject_non_finite_coordinates_and_altitudes() {
        assert_eq!(
            point(f64::NAN, 0.0),
            Err(GeometryError::InvalidNumber(NumericError::NotFinite))
        );
        assert_eq!(
            Altitude::new(f64::INFINITY, AltitudeReference::MeanSeaLevel),
            Err(GeometryError::InvalidNumber(NumericError::NotFinite))
        );
    }

    #[test]
    fn should_canonicalize_a_closed_polygon_ring() -> Result<(), GeometryError> {
        let first = point(-120.0, 50.0)?;
        let polygon = GeoPolygon::new(vec![
            first,
            point(-119.0, 50.0)?,
            point(-119.0, 51.0)?,
            first,
        ])?;
        assert_eq!(polygon.exterior().len(), 3);
        Ok(())
    }

    #[test]
    fn should_reject_degenerate_polygon_ring() -> Result<(), GeometryError> {
        let result = GeoPolygon::new(vec![point(0.0, 0.0)?, point(1.0, 1.0)?, point(2.0, 2.0)?]);
        assert_eq!(result, Err(GeometryError::DegenerateRing));
        Ok(())
    }

    #[test]
    fn should_reject_self_intersecting_polygon() -> Result<(), GeometryError> {
        let result = GeoPolygon::new(vec![
            point(0.0, 0.0)?,
            point(1.0, 1.0)?,
            point(0.0, 1.0)?,
            point(1.0, 0.0)?,
        ]);
        assert_eq!(result, Err(GeometryError::SelfIntersection));
        Ok(())
    }

    #[test]
    fn should_reject_polygon_with_mixed_coordinate_systems() -> Result<(), GeometryError> {
        let projected = CoordinateReferenceSystem::epsg(3857)?;
        let result = GeoPolygon::new(vec![
            point(0.0, 0.0)?,
            point(1.0, 0.0)?,
            GeoPoint::new(1.0, 1.0, projected, None)?,
        ]);
        assert_eq!(result, Err(GeometryError::MixedCrs));
        Ok(())
    }
}

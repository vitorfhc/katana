use std::cmp::Ordering;

use glam::f32::Vec3;

/// Computes the intersection points between a line segment and an infinite horizontal plane at a given height.
///
/// # Remarks
///
/// - If the intersection point is **not within the bounds of the line segment**, the function returns an empty vector.
/// - If the line segment is **parallel to the plane**, the function returns the two end points of the line segment.
/// - If the line segment **intersects the plane**, the function returns the intersection point.
///
/// # Arguments
///
/// * `line` - An array containing the start and end points of the line segment.
/// * `current_layer_height` - The height of the plane at which to compute the intersections.
///
/// # Returns
///
/// A vector containing the intersection points between the line segment and the plane.
fn slice_segment(line: &[Vec3; 2], current_layer_height: f32) -> Vec<Vec3> {
    let line_direction = line[1] - line[0];
    let mut intersections = Vec::new();

    let is_parallel = line_direction.y.total_cmp(&0.0) == Ordering::Equal;
    let same_height = current_layer_height.total_cmp(&line[0].y) == Ordering::Equal;

    if is_parallel && same_height {
        intersections.push(Vec3::new(line[0].x, current_layer_height, line[0].z));
        intersections.push(Vec3::new(line[1].x, current_layer_height, line[1].z));
    } else if !is_parallel {
        let t = (current_layer_height - line[0].y) / line_direction.y;
        if (0.0..=1.0).contains(&t) {
            let intersection = line[0] + line_direction * t;
            intersections.push(Vec3::new(
                intersection.x,
                current_layer_height,
                intersection.z,
            ));
        }
    }

    intersections
}

/// Computes the intersection points between a triangle and an infinite horizontal plane at a given height.
/// The function decomposes the triangle into three line segments and computes the intersection points for each segment.
///
/// # Remarks
///
/// - If the triangle is **parallel to the plane**, the function returns the three vertices of the triangle.
/// - If the triangle **intersects the plane**, the function returns the intersection points.
/// - If the triangle is **completely above or below the plane**, the function returns an empty vector.
///
/// # Arguments
///
/// * `triangle` - An array containing the three vertices of the triangle.
/// * `current_layer_height` - The height of the plane at which to compute the intersections.
///
/// # Returns
///
/// A vector containing the intersection points between the triangle and the plane.
fn slice_triangle(triangle: &[Vec3; 3], current_layer_height: f32) -> Vec<Vec3> {
    let mut intersections = Vec::new();

    let min_y = triangle.iter().map(|v| v.y).fold(f32::INFINITY, f32::min);
    let max_y = triangle
        .iter()
        .map(|v| v.y)
        .fold(f32::NEG_INFINITY, f32::max);
    if current_layer_height < min_y || current_layer_height > max_y {
        return intersections;
    }

    for curr_ind in 0..3 {
        let next_ind = (curr_ind + 1) % 3;
        let line = [triangle[curr_ind], triangle[next_ind]];
        let segment_intersections = slice_segment(&line, current_layer_height);
        intersections.extend(segment_intersections);
    }

    intersections.sort_by(|a, b| compare_by_xyz(a, b));
    intersections.dedup_by(|a, b| a.abs_diff_eq(*b, f32::EPSILON));

    intersections
}

/// Compares two `Vec3` points by their x, y, and z coordinates with a given maximum absolute difference.
///
/// # Arguments
///
/// * `a` - The first `Vec3` point to compare.
/// * `b` - The second `Vec3` point to compare.
/// * `max_abs_diff` - The maximum absolute difference allowed for the coordinates.
///
/// # Returns
///
/// An `Ordering` value indicating the relationship between the two points.
fn compare_by_xyz(a: &Vec3, b: &Vec3) -> Ordering {
    if a.x.total_cmp(&b.x) != Ordering::Equal {
        return a.x.total_cmp(&b.x);
    } else if a.y.total_cmp(&b.y) != Ordering::Equal {
        return a.y.total_cmp(&b.y);
    } else if a.z.total_cmp(&b.z) != Ordering::Equal {
        return a.z.total_cmp(&b.z);
    } else {
        return Ordering::Equal;
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    /// Test for the `slice_segment` function when the segment is orthogonal to the current layer height.
    #[test]
    fn test_slice_segment_orthogonal() {
        let line = [Vec3::ZERO, Vec3::Y];
        let current_layer_height = 0.5;
        let intersections = slice_segment(&line, current_layer_height);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0], Vec3::new(0.0, 0.5, 0.0));
    }

    /// Test for the `slice_segment` function when the segment is parallel to the current layer height.
    #[test]
    fn test_slice_segment_parallel() {
        let line = [Vec3::ZERO, Vec3::X];
        let current_layer_height = 0.0;
        let intersections = slice_segment(&line, current_layer_height);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(intersections[1], Vec3::new(1.0, 0.0, 0.0));
    }

    /// Test for the `slice_segment` function when the segment intersects (but is not orthogonal or parallel to) the current layer height.
    #[test]
    fn test_slice_segment_intersection() {
        let line = [Vec3::ZERO, Vec3::ONE];
        let current_layer_height = 0.5;
        let intersections = slice_segment(&line, current_layer_height);
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0], Vec3::new(0.5, 0.5, 0.5));
    }

    /// Test for the `slice_segment` function when the segment does not intersect the current layer height.
    #[test]
    fn test_slice_segment_no_intersection() {
        let line = [Vec3::ZERO, Vec3::X];
        let current_layer_height = 1.5;
        let intersections = slice_segment(&line, current_layer_height);
        assert_eq!(intersections.len(), 0);

        let line = [Vec3::ZERO, Vec3::Y];
        let current_layer_height = 1.5;
        let intersections = slice_segment(&line, current_layer_height);
        assert_eq!(intersections.len(), 0);
    }

    /// Test for the `slice_triangle` function when the triangle face is parallel to the current layer height.
    #[test]
    fn test_slice_triangle_parallel() {
        let triangle = [Vec3::ZERO, Vec3::X, Vec3::Z];
        let current_layer_height = 0.0;
        let intersections = slice_triangle(&triangle, current_layer_height);
        assert_eq!(intersections.len(), 3);
        assert_eq!(intersections[0], Vec3::ZERO);
        assert_eq!(intersections[1], Vec3::Z);
        assert_eq!(intersections[2], Vec3::X);
    }

    /// Test for the `slice_triangle` function when the triangle face intersects
    /// the current layer height orthogonally.
    #[test]
    fn test_slice_triangle_orthogonal() {
        let triangle = [Vec3::ZERO, Vec3::X, Vec3::Y];
        let current_layer_height = 0.5;
        let intersections = slice_triangle(&triangle, current_layer_height);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], Vec3::new(0.0, 0.5, 0.0));
        assert_eq!(intersections[1], Vec3::new(0.5, 0.5, 0.0));

        let triangle = [Vec3::ZERO, Vec3::X, Vec3::new(0.5, 1.0, 0.0)];
        let current_layer_height = 0.5;
        let intersections = slice_triangle(&triangle, current_layer_height);
        assert_eq!(intersections.len(), 2);
        assert_eq!(intersections[0], Vec3::new(0.25, 0.5, 0.0));
        assert_eq!(intersections[1], Vec3::new(0.75, 0.5, 0.0));
    }

    #[test]
    fn test_compare_by_xyz() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 0.0, 0.0);
        assert_eq!(compare_by_xyz(&a, &b), Ordering::Equal);

        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 0.0, 0.0001);
        assert_eq!(compare_by_xyz(&a, &b), Ordering::Less);

        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 0.0, 0.0001);
        assert_eq!(compare_by_xyz(&a, &b), Ordering::Less);

        let a = Vec3::new(0.0, 0.0, 1.0);
        let b = Vec3::new(0.0, 0.0001, 1.0);
        assert_eq!(compare_by_xyz(&a, &b), Ordering::Less);

        let a = Vec3::new(0.0, 1.0, 0.0);
        let b = Vec3::new(0.0, 0.0, 0.0);
        assert_eq!(compare_by_xyz(&a, &b), Ordering::Greater);

        let a = Vec3::new(1.0, 1.0, 1.0);
        let b = Vec3::new(0.0, 1.0, 1.0);
        assert_eq!(compare_by_xyz(&a, &b), Ordering::Greater);
    }
}

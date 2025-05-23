use std::slice::IterMut;

use tracing::debug;

use super::{context::InterpolationCtx, Interpolater, InterpolationInner, SupportingPoint};

#[derive(Debug)]
pub struct LinearInterpolation {
    ctx: InterpolationCtx,
}

impl InterpolationInner for LinearInterpolation {
    fn new(supporting_points: impl IntoIterator<Item = super::SupportingPoint>) -> Self {
        let ctx = InterpolationCtx::new(supporting_points);

        Self { ctx }
    }
}

impl Interpolater for LinearInterpolation {
    fn interpolate(&mut self) -> &[f32] {
        for point in self.ctx.supporting_points.iter() {
            self.ctx.bar_values[point.x] = point.y;
        }

        debug!("{:?}", self.ctx);

        for section in self.ctx.sections.iter() {
            let left = &self.ctx.supporting_points[section.left_supporting_point_idx];
            let right = &self.ctx.supporting_points[section.left_supporting_point_idx + 1];

            let amount = section.amount;
            for interpolate_idx in 0..amount {
                let t = (interpolate_idx + 1) as f32 / (amount + 1) as f32;

                let idx = left.x + interpolate_idx + 1;
                self.ctx.bar_values[idx] = t * right.y + (1. - t) * left.y;
            }
        }

        &self.ctx.bar_values
    }

    fn supporting_points_mut(&mut self) -> IterMut<'_, SupportingPoint> {
        self.ctx.supporting_points.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_supporting_points_and_zero_sections() {
        let mut interpolator = LinearInterpolation::new([]);
        assert_eq!(interpolator.interpolate(), []);
    }

    #[test]
    fn one_supporting_point_and_zero_sections() {
        let supporting_points = [SupportingPoint { x: 0, y: 0.5 }];

        let mut interpolator = LinearInterpolation::new(supporting_points);

        assert_eq!(interpolator.interpolate(), &[0.5]);
    }

    #[test]
    fn two_supporting_points_and_one_section() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 4, y: 1.0 },
        ];

        let mut interpolator = LinearInterpolation::new(supporting_points);

        assert_eq!(interpolator.interpolate(), &[0.0, 0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn three_supporting_points_and_one_section() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 2, y: 1.0 },
            SupportingPoint { x: 3, y: 0.0 },
        ];

        let mut interpolator = LinearInterpolation::new(supporting_points);

        assert_eq!(interpolator.interpolate(), &[0.0, 0.5, 1.0, 0.0]);
    }

    #[test]
    fn three_supporting_points_and_two_sections() {
        let supporting_points = [
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 2, y: 1.0 },
            SupportingPoint { x: 6, y: 0.0 },
        ];

        let mut interpolator = LinearInterpolation::new(supporting_points);

        println!("{:?}", interpolator.ctx);

        assert_eq!(
            interpolator.interpolate(),
            &[0.0, 0.5, 1.0, 0.75, 0.5, 0.25, 0.0],
        );
    }
}

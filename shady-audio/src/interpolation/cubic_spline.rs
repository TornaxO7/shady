use nalgebra::{Cholesky, DMatrix, DVector, Dyn};

use super::{context::InterpolationCtx, Interpolater, InterpolationInstantiator};

type Width = usize;

#[derive(Debug, Clone)]
pub struct CubicSplineInterpolation {
    values: Box<[f32]>,
    ctx: InterpolationCtx,

    section_widths: Box<[Width]>,

    matrix: Cholesky<f32, Dyn>,
    gradients: Box<[f32]>,
    gradient_diffs: Box<[f32]>,
}

impl InterpolationInstantiator for CubicSplineInterpolation {
    fn new(supporting_points: impl IntoIterator<Item = super::SupportingPoint>) -> Self {
        let ctx = InterpolationCtx::new(supporting_points);
        let values = vec![0f32; ctx.total_amount_entries()].into_boxed_slice();

        let section_widths = {
            let mut section_width = Vec::with_capacity(ctx.supporting_points.len() - 1);

            for (i, right) in ctx.supporting_points[1..].iter().enumerate() {
                let left = &ctx.supporting_points[i];

                let width = right.x - left.x;
                section_width.push(width);
            }

            section_width.into_boxed_slice()
        };
        let amount_sections = section_widths.len();

        let matrix = get_matrix(&section_widths);
        let gradients = vec![0f32; amount_sections].into_boxed_slice();
        let gradient_diffs = vec![0f32; amount_sections].into_boxed_slice();

        Self {
            values,
            ctx,
            section_widths,
            matrix,
            gradients,
            gradient_diffs,
        }
    }
}

impl Interpolater for CubicSplineInterpolation {
    fn interpolate(&mut self) -> &[f32] {
        for point in self.ctx.supporting_points.iter() {
            self.values[point.x] = point.y;
        }

        // == preparation ==
        // update gradients
        for (i, next) in self.ctx.supporting_points[1..].iter().enumerate() {
            let prev = &self.ctx.supporting_points[i];
            self.gradients[i] = (prev.y - next.y) / (prev.x as f32 - next.x as f32);
        }

        // update gradient diffs
        {
            self.gradient_diffs[0] = self.gradients[0];

            for (prev_idx, &next) in self.gradients[1..(self.gradients.len() - 1)]
                .iter()
                .enumerate()
            {
                let prev = self.gradients[prev_idx];
                self.gradient_diffs[prev_idx + 1] = next - prev;
            }

            *self.gradient_diffs.last_mut().unwrap() = -self.gradients.last().unwrap();
        }

        // solve gamma
        let gammas = self
            .matrix
            .solve(&DVector::from_column_slice(&self.gradient_diffs));

        // == interpolation ==
        for section in self.ctx.sections.iter() {
            let n = section.left_supporting_point_idx + 1;

            let left = &self.ctx.supporting_points[n - 1];
            let right = &self.ctx.supporting_points[n];

            let prev_gamma = gammas[n - 1];
            let next_gamma = gammas[n];

            let gradient = self.gradients[n - 1];
            let section_width = self.section_widths[n - 1];

            let amount = section.amount;
            for interpolated_idx in 0..amount {
                let bar_idx = interpolated_idx + 1 + left.x;
                let x = bar_idx as f32;

                let interpolated_value = left.y
                    + (x - left.x as f32) * gradient
                    + ((x - left.x as f32) * (x - right.x as f32)) / (6. * section_width as f32)
                        * ((prev_gamma + 2. * next_gamma) * (x - left.x as f32)
                            - (2. * prev_gamma + next_gamma) * (x - right.x as f32));

                self.values[bar_idx] = interpolated_value;
            }
        }

        &self.values
    }

    fn total_amount_entries(&self) -> usize {
        self.ctx.total_amount_entries()
    }

    fn supporting_points_mut(&mut self) -> std::slice::IterMut<'_, super::SupportingPoint> {
        self.ctx.supporting_points.iter_mut()
    }
}

fn get_matrix(section_widths: &[usize]) -> Cholesky<f32, Dyn> {
    let amount_widths = section_widths.len();
    let mut matrix = DMatrix::zeros(amount_widths, amount_widths);

    // add first row
    {
        let mut first_row = matrix.row_mut(0);
        first_row[0] = 2. * section_widths[0] as f32;
        first_row[1] = section_widths[0] as f32;
    }

    // add rows in between
    {
        let mut offset = 0;
        for row_idx in 1..(amount_widths - 1) {
            let mut row = matrix.row_mut(row_idx);

            row[offset] = section_widths[offset] as f32;
            row[offset + 1] = 2. * (section_widths[offset] + section_widths[offset + 1]) as f32;
            row[offset + 2] = section_widths[offset + 1] as f32;

            offset += 1;
        }
    }

    // add last row
    {
        let mut last_row = matrix.row_mut(amount_widths - 1);

        last_row[amount_widths - 2] = section_widths[amount_widths - 1] as f32;
        last_row[amount_widths - 1] = 2. * section_widths[amount_widths - 1] as f32;
    }

    ((1. / 6.) * matrix.clone())
        .cholesky()
        .expect(&format!("Hold up! Looks like my numeric knowledge isn't really numericing ;-----;\nThe matrix which got calculated is: {}", matrix))
}

#[cfg(test)]
mod tests {
    use crate::interpolation::SupportingPoint;

    use super::*;

    #[test]
    fn equidistant_supporting_points() {
        const AMOUNT_POINTS: usize = 10;
        const AMOUNT_SECTIONS: usize = AMOUNT_POINTS - 1;

        let supporting_points = [
            SupportingPoint { x: 0, y: 0.0 },
            SupportingPoint { x: 1, y: 0.0 },
            SupportingPoint { x: 2, y: 0.0 },
            SupportingPoint { x: 3, y: 0.0 },
            SupportingPoint { x: 4, y: 0.0 },
            SupportingPoint { x: 5, y: 0.0 },
            SupportingPoint { x: 6, y: 0.0 },
            SupportingPoint { x: 7, y: 0.0 },
            SupportingPoint { x: 8, y: 0.0 },
            SupportingPoint { x: 9, y: 0.0 },
        ];

        let interpolator = CubicSplineInterpolation::new(supporting_points);

        assert_eq!(interpolator.section_widths.as_ref(), &[1; AMOUNT_SECTIONS]);

        // check matrix
        {
            #[rustfmt::skip]
            let expected_matrix = {
                let mut matrix = DMatrix::from_row_slice(
                AMOUNT_SECTIONS,
                AMOUNT_SECTIONS,
                &[
                            2., 1., 0., 0., 0., 0., 0., 0., 0.,
                            1., 4., 1., 0., 0., 0., 0., 0., 0.,
                            0., 1., 4., 1., 0., 0., 0., 0., 0.,
                            0., 0., 1., 4., 1., 0., 0., 0., 0.,
                            0., 0., 0., 1., 4., 1., 0., 0., 0.,
                            0., 0., 0., 0., 1., 4., 1., 0., 0.,
                            0., 0., 0., 0., 0., 1., 4., 1., 0.,
                            0., 0., 0., 0., 0., 0., 1., 4., 1.,
                            0., 0., 0., 0., 0., 0., 0., 1., 2.,
                        ]);

                matrix *= 1. / 6.;
                matrix.cholesky().unwrap()
            };

            assert_eq!(interpolator.matrix.l(), expected_matrix.l());
        }
    }
}
